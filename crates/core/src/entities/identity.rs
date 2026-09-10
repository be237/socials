use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identity {
    pub id: Uuid,
    pub contact_id: Uuid,
    pub account_id: Uuid,
    pub platform_user_id: String,
    pub platform_username: Option<String>,
    pub created_at: DateTime<Utc>,
}
