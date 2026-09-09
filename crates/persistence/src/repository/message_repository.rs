use async_trait::async_trait;
use sqlx::Row;
use sqlx::SqlitePool;
use uuid::Uuid;
use socials_core::entities::message::{Message, MessageType, MessageStatus};
use socials_core::repositories::{MessageRepository, Error};

pub struct SqliteMessageRepository {
    pool: SqlitePool,
}

impl SqliteMessageRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

fn parse_message_type(s: &str) -> MessageType {
    match s {
        "text" => MessageType::Text,
        "image" => MessageType::Image,
        "file" => MessageType::File,
        "voice" => MessageType::Voice,
        "video" => MessageType::Video,
        "sticker" => MessageType::Sticker,
        _ => MessageType::Other,
    }
}

fn parse_message_status(s: &str) -> MessageStatus {
    match s {
        "pending" => MessageStatus::Pending,
        "sent" => MessageStatus::Sent,
        "delivered" => MessageStatus::Delivered,
        "read" => MessageStatus::Read,
        "failed" => MessageStatus::Failed,
        _ => MessageStatus::Pending,
    }
}

fn parse_message(row: sqlx::sqlite::SqliteRow) -> Message {
    let msg_type_str: String = row.get("message_type");
    let status_str: String = row.get("status");
    let edited_at: Option<String> = row.get("edited_at");
    let reply_to: Option<String> = row.get("reply_to");

    Message {
        id: Uuid::parse_str(row.get::<&str, _>("id")).unwrap(),
        conversation_id: Uuid::parse_str(row.get::<&str, _>("conversation_id")).unwrap(),
        sender_id: Uuid::parse_str(row.get::<&str, _>("sender_id")).unwrap(),
        content: row.get("content"),
        message_type: parse_message_type(&msg_type_str),
        status: parse_message_status(&status_str),
        created_at: chrono::DateTime::parse_from_rfc3339(row.get::<&str, _>("created_at"))
            .unwrap()
            .with_timezone(&chrono::Utc),
        edited_at: edited_at.and_then(|s| {
            chrono::DateTime::parse_from_rfc3339(&s)
                .ok()
                .map(|dt| dt.with_timezone(&chrono::Utc))
        }),
        reply_to: reply_to.and_then(|s| Uuid::parse_str(&s).ok()),
        connector_id: row.get("connector_id"),
        platform_message_id: row.get("platform_message_id"),
    }
}

#[async_trait]
impl MessageRepository for SqliteMessageRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Message>, Error> {
        let row = sqlx::query("SELECT * FROM messages WHERE id = ?")
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(parse_message))
    }

    async fn find_by_conversation(&self, conversation_id: Uuid) -> Result<Vec<Message>, Error> {
        let rows = sqlx::query("SELECT * FROM messages WHERE conversation_id = ? ORDER BY created_at ASC")
            .bind(conversation_id.to_string())
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.into_iter().map(parse_message).collect())
    }

    async fn create(&self, message: &Message) -> Result<Message, Error> {
        let msg_type = match message.message_type {
            MessageType::Text => "text",
            MessageType::Image => "image",
            MessageType::File => "file",
            MessageType::Voice => "voice",
            MessageType::Video => "video",
            MessageType::Sticker => "sticker",
            MessageType::Other => "other",
        };

        let status = match message.status {
            MessageStatus::Pending => "pending",
            MessageStatus::Sent => "sent",
            MessageStatus::Delivered => "delivered",
            MessageStatus::Read => "read",
            MessageStatus::Failed => "failed",
        };

        sqlx::query("INSERT INTO messages (id, conversation_id, sender_id, content, message_type, status, created_at, edited_at, reply_to, connector_id, platform_message_id) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)")
            .bind(message.id.to_string())
            .bind(message.conversation_id.to_string())
            .bind(message.sender_id.to_string())
            .bind(&message.content)
            .bind(msg_type)
            .bind(status)
            .bind(message.created_at.to_rfc3339())
            .bind(message.edited_at.map(|dt| dt.to_rfc3339()))
            .bind(message.reply_to.map(|id| id.to_string()))
            .bind(&message.connector_id)
            .bind(&message.platform_message_id)
            .execute(&self.pool)
            .await?;
        Ok(message.clone())
    }

    async fn update(&self, message: &Message) -> Result<Message, Error> {
        let msg_type = match message.message_type {
            MessageType::Text => "text",
            MessageType::Image => "image",
            MessageType::File => "file",
            MessageType::Voice => "voice",
            MessageType::Video => "video",
            MessageType::Sticker => "sticker",
            MessageType::Other => "other",
        };

        let status = match message.status {
            MessageStatus::Pending => "pending",
            MessageStatus::Sent => "sent",
            MessageStatus::Delivered => "delivered",
            MessageStatus::Read => "read",
            MessageStatus::Failed => "failed",
        };

        sqlx::query("UPDATE messages SET content = ?, message_type = ?, status = ?, edited_at = ?, reply_to = ? WHERE id = ?")
            .bind(&message.content)
            .bind(msg_type)
            .bind(status)
            .bind(message.edited_at.map(|dt| dt.to_rfc3339()))
            .bind(message.reply_to.map(|id| id.to_string()))
            .bind(message.id.to_string())
            .execute(&self.pool)
            .await?;
        Ok(message.clone())
    }

    async fn delete(&self, id: Uuid) -> Result<(), Error> {
        sqlx::query("DELETE FROM messages WHERE id = ?")
            .bind(id.to_string())
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
