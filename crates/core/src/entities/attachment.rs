use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AttachmentType {
    Image,
    File,
    Voice,
    Video,
    Sticker,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    pub id: Uuid,
    pub message_id: Uuid,
    pub attachment_type: AttachmentType,
    pub filename: String,
    pub url: Option<String>,
    pub size: Option<u64>,
    pub mime_type: Option<String>,
    pub created_at: DateTime<Utc>,
}
