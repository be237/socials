use reqwest::Client;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::Utc;

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

        let resp: TgResponse<TgMe> = client
            .get(format!("https://api.telegram.org/bot{}/getMe", token))
            .send()
            .await?
            .json()
            .await?;

        let me = resp.result;

        println!("Telegram bot connected: @{}", me.username.unwrap_or_default());

        Ok(Self {
            token,
            client,
            bot_id: me.id,
            offset: 0,
        })
    }

    pub async fn poll_updates(&mut self, core: &socials_core::services::CoreService) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let url = format!(
            "https://api.telegram.org/bot{}/getUpdates?offset={}&timeout=30",
            self.token, self.offset
        );

        let resp: TgResponse<Vec<TgUpdate>> = self.client.get(&url).send().await?.json().await?;

        if !resp.ok {
            return Ok(());
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
        let sender_name = format!("{} {}", msg.from.first_name, msg.from.last_name.as_deref().unwrap_or(""));

        // Find or create conversation for this chat
        let conv = self.find_or_create_conversation(core, chat_id, &msg.chat).await?;

        let now = Utc::now();
        let message = socials_core::entities::message::Message {
            id: Uuid::new_v4(),
            conversation_id: conv.id,
            sender_id: Uuid::new_v4(),
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
    ) -> Result<socials_core::entities::conversation::Conversation, Box<dyn std::error::Error + Send + Sync>> {
        let title_prefix = format!("TG:{}:", chat_id);
        if let Ok(Some(existing)) = core.find_conversation_by_title_prefix(&title_prefix).await {
            return Ok(existing);
        }

        let now = Utc::now();
        let title = format!(
            "TG:{}:{}",
            chat_id,
            chat.title.as_deref().unwrap_or(&chat.chat_type)
        );

        let conv = socials_core::entities::conversation::Conversation {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            conversation_type: socials_core::entities::conversation::ConversationType::Private,
            title,
            created_at: now,
            updated_at: now,
            last_message_at: Some(now),
        };

        let created = core.create_conversation(&conv).await?;
        Ok(created)
    }

    pub async fn send_message(&self, chat_id: i64, text: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("https://api.telegram.org/bot{}/sendMessage", self.token);

        self.client.post(&url)
            .json(&TgSendMessageRequest { chat_id, text: text.to_string() })
            .send()
            .await?;

        Ok(())
    }

    pub fn bot_id(&self) -> i64 {
        self.bot_id
    }

    pub fn extract_chat_id(title: &str) -> Option<i64> {
        if title.starts_with("TG:") {
            let parts: Vec<&str> = title.split(':').collect();
            if parts.len() >= 2 {
                return parts[1].parse().ok();
            }
        }
        None
    }
}
