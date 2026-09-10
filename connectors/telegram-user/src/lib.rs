mod session;

use std::sync::Arc;
use tokio::sync::RwLock;
use grammers_client::{Client, SignInError};
use grammers_mtsender::SenderPool;

pub use session::FileSession;

const API_ID: i32 = 32939;
const API_HASH: &str = "ec7e8f309b3108f8e6e7e0e2c0e7b2d8";

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum AuthState {
    Disconnected,
    WaitingCode { phone: String },
    Connected { user_id: i64, username: String },
}

pub struct TelegramUserClient {
    client: Option<Client>,
    pub auth_state: Arc<RwLock<AuthState>>,
    session_path: std::path::PathBuf,
}

impl TelegramUserClient {
    pub fn new(data_dir: &std::path::Path) -> Self {
        let session_path = data_dir.join("telegram_session.json");
        Self {
            client: None,
            auth_state: Arc::new(RwLock::new(AuthState::Disconnected)),
            session_path,
        }
    }

    pub async fn connect(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let session = Arc::new(FileSession::new(self.session_path.clone())?);

        let SenderPool { runner, handle, .. } = SenderPool::new(session, API_ID);
        let client = Client::new(handle);

        tokio::spawn(runner.run());

        self.client = Some(client);

        if self.is_authorized().await? {
            let me = self.client.as_ref().unwrap().get_me().await?;
            let mut state = self.auth_state.write().await;
            *state = AuthState::Connected {
                user_id: me.id().bare_id_unchecked(),
                username: me.username().map(|s| s.to_string()).unwrap_or_default(),
            };
        }

        Ok(())
    }

    pub async fn is_authorized(&self) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        match &self.client {
            Some(client) => Ok(client.is_authorized().await?),
            None => Ok(false),
        }
    }

    pub async fn request_code(
        &mut self,
        phone: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = self.client.as_ref()
            .ok_or("Client not connected. Call connect() first.")?;

        client.request_login_code(phone, API_HASH).await?;

        let mut state = self.auth_state.write().await;
        *state = AuthState::WaitingCode { phone: phone.to_string() };

        Ok(())
    }

    pub async fn sign_in(
        &mut self,
        code: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let client = self.client.as_ref()
            .ok_or("Client not connected. Call connect() first.")?;

        let state = self.auth_state.read().await;
        let phone = match &*state {
            AuthState::WaitingCode { phone } => phone.clone(),
            _ => return Err("No code requested. Call request_code first.".into()),
        };
        drop(state);

        let token = client.request_login_code(&phone, API_HASH).await?;

        match client.sign_in(&token, code).await {
            Ok(user) => {
                let mut state = self.auth_state.write().await;
                *state = AuthState::Connected {
                    user_id: user.id().bare_id_unchecked(),
                    username: user.username().map(|s| s.to_string()).unwrap_or_default(),
                };

                let username = user.username().unwrap_or("unknown").to_string();
                let user_id = user.id().bare_id_unchecked();
                Ok(format!("Connected as {} (id: {})", username, user_id))
            }
            Err(SignInError::PasswordRequired(_)) => {
                Err("Two-factor authentication required. Not yet supported.".into())
            }
            Err(SignInError::InvalidCode) => {
                Err("Invalid code. Please try again.".into())
            }
            Err(e) => Err(format!("Sign in failed: {}", e).into()),
        }
    }

    pub async fn disconnect(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(client) = self.client.take() {
            client.disconnect();
        }

        let mut state = self.auth_state.write().await;
        *state = AuthState::Disconnected;

        Ok(())
    }

    pub async fn get_dialogs(
        &self,
    ) -> Result<Vec<DialogInfo>, Box<dyn std::error::Error + Send + Sync>> {
        let client = self.client.as_ref()
            .ok_or("Client not connected.")?;

        let mut dialogs = Vec::new();
        let mut iter = client.iter_dialogs();

        while let Some(dialog) = iter.next().await? {
            let peer = dialog.peer();
            let chat_id = peer.id().bare_id_unchecked();
            let title = peer.name().unwrap_or("Unknown").to_string();

            let last_message = dialog.last_message.as_ref().map(|m| {
                m.text().to_string()
            });

            dialogs.push(DialogInfo {
                chat_id,
                title,
                last_message,
            });
        }

        Ok(dialogs)
    }

    pub async fn send_message(
        &self,
        chat_id: i64,
        text: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = self.client.as_ref()
            .ok_or("Client not connected.")?;

        let mut iter = client.iter_dialogs();
        while let Some(dialog) = iter.next().await? {
            let p = dialog.peer();
            if p.id().bare_id_unchecked() == chat_id {
                if let Some(peer_ref) = p.to_ref().await? {
                    client.send_message(peer_ref, text).await?;
                    return Ok(());
                }
            }
        }

        Err("Chat not found in dialogs".into())
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DialogInfo {
    pub chat_id: i64,
    pub title: String,
    pub last_message: Option<String>,
}
