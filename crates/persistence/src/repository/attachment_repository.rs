use async_trait::async_trait;
use socials_core::entities::attachment::{Attachment, AttachmentType};
use socials_core::repositories::{AttachmentRepository, Error};
use sqlx::Row;
use sqlx::SqlitePool;
use uuid::Uuid;

pub struct SqliteAttachmentRepository {
    pool: SqlitePool,
}

impl SqliteAttachmentRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

fn parse_attachment_type(s: &str) -> AttachmentType {
    match s {
        "image" => AttachmentType::Image,
        "file" => AttachmentType::File,
        "voice" => AttachmentType::Voice,
        "video" => AttachmentType::Video,
        "sticker" => AttachmentType::Sticker,
        _ => AttachmentType::Other,
    }
}

fn parse_attachment(row: sqlx::sqlite::SqliteRow) -> Attachment {
    let att_type_str: String = row.get("attachment_type");
    let size: Option<i64> = row.get("size");

    Attachment {
        id: Uuid::parse_str(row.get::<&str, _>("id")).unwrap(),
        message_id: Uuid::parse_str(row.get::<&str, _>("message_id")).unwrap(),
        attachment_type: parse_attachment_type(&att_type_str),
        filename: row.get("filename"),
        url: row.get("url"),
        size: size.map(|s| s as u64),
        mime_type: row.get("mime_type"),
        created_at: chrono::DateTime::parse_from_rfc3339(row.get::<&str, _>("created_at"))
            .unwrap()
            .with_timezone(&chrono::Utc),
    }
}

#[async_trait]
impl AttachmentRepository for SqliteAttachmentRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Attachment>, Error> {
        let row = sqlx::query("SELECT * FROM attachments WHERE id = ?")
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(parse_attachment))
    }

    async fn find_by_message_id(&self, message_id: Uuid) -> Result<Vec<Attachment>, Error> {
        let rows = sqlx::query("SELECT * FROM attachments WHERE message_id = ?")
            .bind(message_id.to_string())
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.into_iter().map(parse_attachment).collect())
    }

    async fn create(&self, attachment: &Attachment) -> Result<Attachment, Error> {
        let att_type = match attachment.attachment_type {
            AttachmentType::Image => "image",
            AttachmentType::File => "file",
            AttachmentType::Voice => "voice",
            AttachmentType::Video => "video",
            AttachmentType::Sticker => "sticker",
            AttachmentType::Other => "other",
        };

        sqlx::query("INSERT INTO attachments (id, message_id, attachment_type, filename, url, size, mime_type, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)")
            .bind(attachment.id.to_string())
            .bind(attachment.message_id.to_string())
            .bind(att_type)
            .bind(&attachment.filename)
            .bind(&attachment.url)
            .bind(attachment.size.map(|s| s as i64))
            .bind(&attachment.mime_type)
            .bind(attachment.created_at.to_rfc3339())
            .execute(&self.pool)
            .await?;
        Ok(attachment.clone())
    }

    async fn delete(&self, id: Uuid) -> Result<(), Error> {
        sqlx::query("DELETE FROM attachments WHERE id = ?")
            .bind(id.to_string())
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
