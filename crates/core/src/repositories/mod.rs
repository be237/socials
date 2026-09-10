use crate::entities::account::Account;
use crate::entities::attachment::Attachment;
use crate::entities::contact::Contact;
use crate::entities::conversation::Conversation;
use crate::entities::identity::Identity;
use crate::entities::message::Message;
use crate::entities::user::User;
use async_trait::async_trait;
use uuid::Uuid;

pub type Error = Box<dyn std::error::Error + Send + Sync>;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, Error>;
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, Error>;
    async fn create(&self, user: &User) -> Result<User, Error>;
    async fn update(&self, user: &User) -> Result<User, Error>;
    async fn delete(&self, id: Uuid) -> Result<(), Error>;
}

#[async_trait]
pub trait AccountRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Account>, Error>;
    async fn find_by_user_id(&self, user_id: Uuid) -> Result<Vec<Account>, Error>;
    async fn find_by_connector(
        &self,
        user_id: Uuid,
        connector: &str,
    ) -> Result<Vec<Account>, Error>;
    async fn create(&self, account: &Account) -> Result<Account, Error>;
    async fn update(&self, account: &Account) -> Result<Account, Error>;
    async fn delete(&self, id: Uuid) -> Result<(), Error>;
}

#[async_trait]
pub trait ContactRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Contact>, Error>;
    async fn find_by_user_id(&self, user_id: Uuid) -> Result<Vec<Contact>, Error>;
    async fn create(&self, contact: &Contact) -> Result<Contact, Error>;
    async fn update(&self, contact: &Contact) -> Result<Contact, Error>;
    async fn delete(&self, id: Uuid) -> Result<(), Error>;
}

#[async_trait]
pub trait IdentityRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Identity>, Error>;
    async fn find_by_contact_id(&self, contact_id: Uuid) -> Result<Vec<Identity>, Error>;
    async fn find_by_account_id(&self, account_id: Uuid) -> Result<Vec<Identity>, Error>;
    async fn create(&self, identity: &Identity) -> Result<Identity, Error>;
    async fn delete(&self, id: Uuid) -> Result<(), Error>;
}

#[async_trait]
pub trait ConversationRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Conversation>, Error>;
    async fn find_by_user_id(&self, user_id: Uuid) -> Result<Vec<Conversation>, Error>;
    async fn list_all(&self) -> Result<Vec<Conversation>, Error>;
    async fn find_by_title_prefix(&self, prefix: &str) -> Result<Option<Conversation>, Error>;
    async fn create(&self, conversation: &Conversation) -> Result<Conversation, Error>;
    async fn update(&self, conversation: &Conversation) -> Result<Conversation, Error>;
    async fn delete(&self, id: Uuid) -> Result<(), Error>;
}

#[async_trait]
pub trait MessageRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Message>, Error>;
    async fn find_by_conversation(&self, conversation_id: Uuid) -> Result<Vec<Message>, Error>;
    async fn create(&self, message: &Message) -> Result<Message, Error>;
    async fn update(&self, message: &Message) -> Result<Message, Error>;
    async fn delete(&self, id: Uuid) -> Result<(), Error>;
}

#[async_trait]
pub trait AttachmentRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Attachment>, Error>;
    async fn find_by_message_id(&self, message_id: Uuid) -> Result<Vec<Attachment>, Error>;
    async fn create(&self, attachment: &Attachment) -> Result<Attachment, Error>;
    async fn delete(&self, id: Uuid) -> Result<(), Error>;
}
