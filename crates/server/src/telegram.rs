use chrono::Utc;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct TgUpdate {
    pub update_id: i64,
    pub message: Option<TgMessage>,
}

#[derive(Debug, Deserialize)]
pub struct TgMessage {
    pub message_id: i64,
    pub chat: TgChat,
    pub from: TgUser,
    pub text: Option<String>,
    pub date: i64,
}

#[derive(Debug, Deserialize)]
pub struct TgChat {
    pub id: i64,
    #[serde(rename = "type")]
    pub chat_type: String,
    pub title: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TgUser {
    pub id: i64,
    pub first_name: String,
    pub last_name: Option<String>,
    pub username: Option<String>,
}

#[derive(Debug, Serialize)]
struct TgSendMessageRequest {
    chat_id: i64,
    text: String,
}

#[derive(Debug, Deserialize)]
pub struct TgSendMessageResponse {
    pub ok: bool,
    pub result: Option<TgMessage>,
}

#[derive(Debug, Deserialize)]
pub struct TgMe {
    pub id: i64,
    pub first_name: String,
    pub username: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TgResponse<T> {
    ok: bool,
    result: T,
}

pub struct TelegramBot {
    token: String,
    client: Client,
    bot_id: i64,
    offset: i64,
}

impl TelegramBot {
    pub async fn new(token: String) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let client = Client::new();

        let response = client
            .get(format!("https://api.telegram.org/bot{}/getMe", token))
            .send()
            .await?
            .error_for_status()?;
        let resp: TgResponse<TgMe> = response.json().await?;
        if !resp.ok {
            return Err("Telegram rejected the bot token".into());
        }

        let me = resp.result;

        println!(
            "Telegram bot connected: @{}",
            me.username.unwrap_or_default()
        );

        Ok(Self {
            token,
            client,
            bot_id: me.id,
            offset: 0,
        })
    }

    pub async fn poll_updates(
        &mut self,
        core: &socials_core::services::CoreService,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let url = format!(
            "https://api.telegram.org/bot{}/getUpdates?offset={}&timeout=30",
            self.token, self.offset
        );

        let response = self.client.get(&url).send().await?.error_for_status()?;
        let resp: TgResponse<Vec<TgUpdate>> = response.json().await?;

        if !resp.ok {
            return Err("Telegram getUpdates returned ok=false".into());
        }

        for update in resp.result {
            self.offset = update.update_id + 1;

            if let Some(msg) = update.message {
                if let Some(ref text) = msg.text {
                    self.handle_message(core, &msg, text).await?;
                }
            }
        }

        Ok(())
    }

    async fn handle_message(
        &self,
        core: &socials_core::services::CoreService,
        msg: &TgMessage,
        text: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let chat_id = msg.chat.id;
        let sender_name = format!(
            "{} {}",
            msg.from.first_name,
            msg.from.last_name.as_deref().unwrap_or("")
        );

        // Find or create conversation for this chat
        let conv = self
            .find_or_create_conversation(core, chat_id, &msg.chat, &msg.from)
            .await?;

        let now = Utc::now();
        let message = socials_core::entities::message::Message {
            id: Uuid::new_v4(),
            conversation_id: conv.id,
            sender_id: Uuid::new_v5(
                &Uuid::NAMESPACE_URL,
                format!("telegram:user:{}", msg.from.id).as_bytes(),
            ),
            content: text.to_string(),
            message_type: socials_core::entities::message::MessageType::Text,
            status: socials_core::entities::message::MessageStatus::Delivered,
            created_at: now,
            edited_at: None,
            reply_to: None,
            connector_id: format!("Telegram:{}", sender_name.trim()),
            platform_message_id: msg.message_id.to_string(),
        };

        core.receive_message(&message).await?;

        // Update conversation last_message_at
        let mut updated_conv = conv;
        updated_conv.last_message_at = Some(now);
        updated_conv.updated_at = now;
        let _ = core.update_conversation(&updated_conv).await;

        println!("Telegram message from {}: {}", sender_name.trim(), text);

        Ok(())
    }

    async fn find_or_create_conversation(
        &self,
        core: &socials_core::services::CoreService,
        chat_id: i64,
        chat: &TgChat,
        sender: &TgUser,
    ) -> Result<
        socials_core::entities::conversation::Conversation,
        Box<dyn std::error::Error + Send + Sync>,
    > {
        let title_prefix = format!("TG:{}:", chat_id);
        if let Ok(Some(existing)) = core.find_conversation_by_title_prefix(&title_prefix).await {
            return Ok(existing);
        }

        let user = core.get_or_create_default_user().await?;
        let now = Utc::now();

        // For private chats, use the sender's real name instead of "private"
        let display_name = if chat.chat_type == "private" {
            if let Some(ref username) = sender.username {
                format!("@{}", username)
            } else {
                let last = sender.last_name.as_deref().unwrap_or("").trim().to_string();
                if last.is_empty() {
                    sender.first_name.clone()
                } else {
                    format!("{} {}", sender.first_name, last)
                }
            }
        } else {
            chat.title.as_deref().unwrap_or(&chat.chat_type).to_string()
        };

        let title = format!("TG:{}:{}", chat_id, display_name);

        let conv = socials_core::entities::conversation::Conversation {
            id: Uuid::new_v4(),
            user_id: user.id,
            conversation_type: socials_core::entities::conversation::ConversationType::Private,
            title,
            created_at: now,
            updated_at: now,
            last_message_at: Some(now),
        };

        let created = core.create_conversation(&conv).await?;
        Ok(created)
    }

    pub async fn send_message(
        &self,
        chat_id: i64,
        text: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("https://api.telegram.org/bot{}/sendMessage", self.token);

        let response = self
            .client
            .post(&url)
            .json(&TgSendMessageRequest {
                chat_id,
                text: text.to_string(),
            })
            .send()
            .await?
            .error_for_status()?;
        let result: TgResponse<TgMessage> = response.json().await?;
        if !result.ok {
            return Err("Telegram rejected the outgoing message".into());
        }

        Ok(())
    }

    pub fn bot_id(&self) -> i64 {
        self.bot_id
    }

    pub fn extract_chat_id(title: &str) -> Option<i64> {
        title
            .strip_prefix("TG:")
            .and_then(|value| value.split_once(':').map(|(id, _)| id))
            .and_then(|id| id.parse().ok())
    }
}
