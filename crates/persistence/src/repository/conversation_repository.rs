use async_trait::async_trait;
use socials_core::entities::conversation::{Conversation, ConversationType};
use socials_core::repositories::{ConversationRepository, Error};
use sqlx::Row;
use sqlx::SqlitePool;
use uuid::Uuid;

pub struct SqliteConversationRepository {
    pool: SqlitePool,
}

impl SqliteConversationRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

fn parse_conversation_type(s: &str) -> ConversationType {
    match s {
        "private" => ConversationType::Private,
        "group" => ConversationType::Group,
        "channel" => ConversationType::Channel,
        "thread" => ConversationType::Thread,
        "email" => ConversationType::Email,
        "bot" => ConversationType::Bot,
        _ => ConversationType::Private,
    }
}

fn parse_conversation(row: sqlx::sqlite::SqliteRow) -> Conversation {
    let conv_type_str: String = row.get("conversation_type");
    let last_message_at: Option<String> = row.get("last_message_at");

    Conversation {
        id: Uuid::parse_str(row.get::<&str, _>("id")).unwrap(),
        user_id: Uuid::parse_str(row.get::<&str, _>("user_id")).unwrap(),
        conversation_type: parse_conversation_type(&conv_type_str),
        title: row.get("title"),
        created_at: chrono::DateTime::parse_from_rfc3339(row.get::<&str, _>("created_at"))
            .unwrap()
            .with_timezone(&chrono::Utc),
        updated_at: chrono::DateTime::parse_from_rfc3339(row.get::<&str, _>("updated_at"))
            .unwrap()
            .with_timezone(&chrono::Utc),
        last_message_at: last_message_at.and_then(|s| {
            chrono::DateTime::parse_from_rfc3339(&s)
                .ok()
                .map(|dt| dt.with_timezone(&chrono::Utc))
        }),
    }
}

#[async_trait]
impl ConversationRepository for SqliteConversationRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Conversation>, Error> {
        let row = sqlx::query("SELECT * FROM conversations WHERE id = ?")
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(parse_conversation))
    }

    async fn find_by_user_id(&self, user_id: Uuid) -> Result<Vec<Conversation>, Error> {
        let rows = sqlx::query(
            "SELECT * FROM conversations WHERE user_id = ? ORDER BY last_message_at DESC",
        )
        .bind(user_id.to_string())
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(parse_conversation).collect())
    }

    async fn list_all(&self) -> Result<Vec<Conversation>, Error> {
        let rows = sqlx::query("SELECT * FROM conversations ORDER BY last_message_at DESC")
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.into_iter().map(parse_conversation).collect())
    }

    async fn find_by_title_prefix(&self, prefix: &str) -> Result<Option<Conversation>, Error> {
        let row = sqlx::query("SELECT * FROM conversations WHERE title LIKE ? LIMIT 1")
            .bind(format!("{}%", prefix))
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(parse_conversation))
    }

    async fn create(&self, conversation: &Conversation) -> Result<Conversation, Error> {
        let conv_type = match conversation.conversation_type {
            ConversationType::Private => "private",
            ConversationType::Group => "group",
            ConversationType::Channel => "channel",
            ConversationType::Thread => "thread",
            ConversationType::Email => "email",
            ConversationType::Bot => "bot",
        };

        sqlx::query("INSERT INTO conversations (id, user_id, conversation_type, title, created_at, updated_at, last_message_at) VALUES (?, ?, ?, ?, ?, ?, ?)")
            .bind(conversation.id.to_string())
            .bind(conversation.user_id.to_string())
            .bind(conv_type)
            .bind(&conversation.title)
            .bind(conversation.created_at.to_rfc3339())
            .bind(conversation.updated_at.to_rfc3339())
            .bind(conversation.last_message_at.map(|dt| dt.to_rfc3339()))
            .execute(&self.pool)
            .await?;
        Ok(conversation.clone())
    }

    async fn update(&self, conversation: &Conversation) -> Result<Conversation, Error> {
        let conv_type = match conversation.conversation_type {
            ConversationType::Private => "private",
            ConversationType::Group => "group",
            ConversationType::Channel => "channel",
            ConversationType::Thread => "thread",
            ConversationType::Email => "email",
            ConversationType::Bot => "bot",
        };

        sqlx::query("UPDATE conversations SET conversation_type = ?, title = ?, updated_at = ?, last_message_at = ? WHERE id = ?")
            .bind(conv_type)
            .bind(&conversation.title)
            .bind(conversation.updated_at.to_rfc3339())
            .bind(conversation.last_message_at.map(|dt| dt.to_rfc3339()))
            .bind(conversation.id.to_string())
            .execute(&self.pool)
            .await?;
        Ok(conversation.clone())
    }

    async fn delete(&self, id: Uuid) -> Result<(), Error> {
        sqlx::query("DELETE FROM conversations WHERE id = ?")
            .bind(id.to_string())
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
