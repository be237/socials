use async_trait::async_trait;
use chrono::Utc;
use socials_core::connectors::{Connector, ConnectorCapabilities};
use socials_core::entities::message::{Message, MessageStatus, MessageType};
use teloxide::prelude::*;
use teloxide::types::{ChatId, Message as TeloxideMessage};
use tokio::sync::Mutex;
use uuid::Uuid;

pub struct TelegramConnector {
    bot: Bot,
    offset: Mutex<i32>,
}

impl TelegramConnector {
    pub fn new(token: &str) -> Self {
        let bot = Bot::new(token);
        Self {
            bot,
            offset: Mutex::new(0),
        }
    }

    fn parse_message_type(msg: &TeloxideMessage) -> MessageType {
        if msg.photo().is_some() {
            MessageType::Image
        } else if msg.video().is_some() {
            MessageType::Video
        } else if msg.voice().is_some() || msg.audio().is_some() {
            MessageType::Voice
        } else if msg.document().is_some() {
            MessageType::File
        } else if msg.sticker().is_some() {
            MessageType::Sticker
        } else {
            MessageType::Text
        }
    }

    fn extract_content(msg: &TeloxideMessage) -> String {
        if let Some(text) = msg.text() {
            text.to_string()
        } else if let Some(photo) = msg.photo() {
            format!("{} photos", photo.len())
        } else if let Some(video) = msg.video() {
            video.file.id.clone()
        } else if let Some(voice) = msg.voice() {
            voice.file.id.clone()
        } else if let Some(doc) = msg.document() {
            doc.file_name
                .clone()
                .unwrap_or_else(|| "document".to_string())
        } else if let Some(sticker) = msg.sticker() {
            sticker.file.id.clone()
        } else {
            "unsupported content".to_string()
        }
    }
}

#[async_trait]
impl Connector for TelegramConnector {
    fn name(&self) -> &str {
        "telegram"
    }

    fn capabilities(&self) -> ConnectorCapabilities {
        ConnectorCapabilities {
            receive_messages: true,
            send_messages: true,
            images: true,
            files: true,
            voice: true,
            reactions: true,
            edit_message: true,
            delete_message: true,
            typing_indicator: true,
            read_receipts: false,
        }
    }

    async fn send_message(&self, message: &Message) -> Result<(), Box<dyn std::error::Error>> {
        let chat_id = ChatId(message.conversation_id.to_string().parse::<i64>()?);

        match message.message_type {
            MessageType::Text => {
                self.bot.send_message(chat_id, &message.content).await?;
            }
            _ => {
                self.bot.send_message(chat_id, &message.content).await?;
            }
        }

        Ok(())
    }

    async fn receive_messages(&self) -> Result<Vec<Message>, Box<dyn std::error::Error>> {
        let offset = *self.offset.lock().await;
        let updates = self
            .bot
            .get_updates()
            .offset(offset)
            .timeout(30)
            .send()
            .await?;
        let mut messages = Vec::new();
        let mut next_offset = offset;

        for update in updates {
            next_offset = update.id.0.saturating_add(1) as i32;
            if let teloxide::types::UpdateKind::Message(message) = update.kind {
                messages.push(Self::from_teloxide_message(message, Uuid::NAMESPACE_URL));
            }
        }

        *self.offset.lock().await = next_offset;
        Ok(messages)
    }
}

impl TelegramConnector {
    pub fn from_teloxide_message(msg: TeloxideMessage, _account_id: Uuid) -> Message {
        let chat_id = msg.chat.id;
        let user_id = msg
            .from
            .as_ref()
            .map(|u| u.id.to_string())
            .unwrap_or_else(|| "unknown".to_string());

        Message {
            id: Uuid::new_v4(),
            conversation_id: Uuid::new_v5(
                &Uuid::NAMESPACE_URL,
                format!("telegram:chat:{}", chat_id).as_bytes(),
            ),
            sender_id: Uuid::new_v5(
                &Uuid::NAMESPACE_URL,
                format!("telegram:user:{}", user_id).as_bytes(),
            ),
            content: Self::extract_content(&msg),
            message_type: Self::parse_message_type(&msg),
            status: MessageStatus::Delivered,
            created_at: Utc::now(),
            edited_at: None,
            reply_to: msg.reply_to_message().map(|_| Uuid::new_v4()),
            connector_id: "telegram".to_string(),
            platform_message_id: msg.id.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connector_name() {
        let bot = Bot::new("test_token");
        let connector = TelegramConnector {
            bot,
            offset: Mutex::new(0),
        };
        assert_eq!(connector.name(), "telegram");
    }

    #[test]
    fn test_capabilities() {
        let bot = Bot::new("test_token");
        let connector = TelegramConnector {
            bot,
            offset: Mutex::new(0),
        };
        let caps = connector.capabilities();

        assert!(caps.receive_messages);
        assert!(caps.send_messages);
        assert!(caps.images);
        assert!(caps.files);
        assert!(caps.voice);
        assert!(caps.reactions);
        assert!(caps.edit_message);
        assert!(caps.delete_message);
        assert!(caps.typing_indicator);
        assert!(!caps.read_receipts);
    }
}
