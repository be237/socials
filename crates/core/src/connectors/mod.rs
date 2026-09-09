use async_trait::async_trait;
use crate::entities::message::Message;

#[async_trait]
pub trait Connector: Send + Sync {
    fn name(&self) -> &str;
    fn capabilities(&self) -> ConnectorCapabilities;
    async fn send_message(&self, message: &Message) -> Result<(), Box<dyn std::error::Error>>;
    async fn receive_messages(&self) -> Result<Vec<Message>, Box<dyn std::error::Error>>;
}

pub struct ConnectorCapabilities {
    pub receive_messages: bool,
    pub send_messages: bool,
    pub images: bool,
    pub files: bool,
    pub voice: bool,
    pub reactions: bool,
    pub edit_message: bool,
    pub delete_message: bool,
    pub typing_indicator: bool,
    pub read_receipts: bool,
}
