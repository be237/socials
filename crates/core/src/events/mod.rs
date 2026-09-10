use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

pub mod bus;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Event {
    // Messages
    MessageReceived {
        message_id: Uuid,
        conversation_id: Uuid,
        sender_id: Uuid,
        connector_id: String,
    },
    MessageSent {
        message_id: Uuid,
        conversation_id: Uuid,
        connector_id: String,
    },
    MessageEdited {
        message_id: Uuid,
        conversation_id: Uuid,
    },
    MessageDeleted {
        message_id: Uuid,
        conversation_id: Uuid,
    },

    // Conversations
    ConversationCreated {
        conversation_id: Uuid,
        user_id: Uuid,
    },
    ConversationUpdated {
        conversation_id: Uuid,
        user_id: Uuid,
    },

    // Contacts
    ContactCreated {
        contact_id: Uuid,
        user_id: Uuid,
    },
    ContactMerged {
        source_contact_id: Uuid,
        target_contact_id: Uuid,
        user_id: Uuid,
    },

    // Accounts
    AccountConnected {
        account_id: Uuid,
        user_id: Uuid,
        connector_name: String,
    },
    AccountDisconnected {
        account_id: Uuid,
        user_id: Uuid,
        connector_name: String,
    },

    // Sync
    SyncStarted {
        account_id: Uuid,
        user_id: Uuid,
    },
    SyncCompleted {
        account_id: Uuid,
        user_id: Uuid,
        messages_synced: u32,
    },
    SyncFailed {
        account_id: Uuid,
        user_id: Uuid,
        error: String,
    },
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Event::MessageReceived { .. } => write!(f, "MESSAGE_RECEIVED"),
            Event::MessageSent { .. } => write!(f, "MESSAGE_SENT"),
            Event::MessageEdited { .. } => write!(f, "MESSAGE_EDITED"),
            Event::MessageDeleted { .. } => write!(f, "MESSAGE_DELETED"),
            Event::ConversationCreated { .. } => write!(f, "CONVERSATION_CREATED"),
            Event::ConversationUpdated { .. } => write!(f, "CONVERSATION_UPDATED"),
            Event::ContactCreated { .. } => write!(f, "CONTACT_CREATED"),
            Event::ContactMerged { .. } => write!(f, "CONTACT_MERGED"),
            Event::AccountConnected { .. } => write!(f, "ACCOUNT_CONNECTED"),
            Event::AccountDisconnected { .. } => write!(f, "ACCOUNT_DISCONNECTED"),
            Event::SyncStarted { .. } => write!(f, "SYNC_STARTED"),
            Event::SyncCompleted { .. } => write!(f, "SYNC_COMPLETED"),
            Event::SyncFailed { .. } => write!(f, "SYNC_FAILED"),
        }
    }
}

pub type EventId = Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope {
    pub id: EventId,
    pub event: Event,
    pub timestamp: DateTime<Utc>,
}

impl EventEnvelope {
    pub fn new(event: Event) -> Self {
        Self {
            id: Uuid::new_v4(),
            event,
            timestamp: Utc::now(),
        }
    }
}
