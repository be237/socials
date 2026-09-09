use crate::entities::user::User;
use crate::entities::account::Account;
use crate::entities::contact::Contact;
use crate::entities::conversation::Conversation;
use crate::entities::message::Message;
use crate::events::{Event, bus::EventBus};
use crate::repositories::{
    UserRepository, AccountRepository, ContactRepository,
    ConversationRepository, MessageRepository, Error,
};
use uuid::Uuid;
use std::sync::Arc;

pub struct CoreService {
    users: Arc<dyn UserRepository>,
    accounts: Arc<dyn AccountRepository>,
    contacts: Arc<dyn ContactRepository>,
    conversations: Arc<dyn ConversationRepository>,
    messages: Arc<dyn MessageRepository>,
    event_bus: Arc<EventBus>,
}

impl CoreService {
    pub fn new(
        users: Arc<dyn UserRepository>,
        accounts: Arc<dyn AccountRepository>,
        contacts: Arc<dyn ContactRepository>,
        conversations: Arc<dyn ConversationRepository>,
        messages: Arc<dyn MessageRepository>,
        event_bus: Arc<EventBus>,
    ) -> Self {
        Self {
            users,
            accounts,
            contacts,
            conversations,
            messages,
            event_bus,
        }
    }

    // User methods
    pub async fn get_user(&self, id: Uuid) -> Result<Option<User>, Error> {
        self.users.find_by_id(id).await
    }

    pub async fn get_user_by_email(&self, email: &str) -> Result<Option<User>, Error> {
        self.users.find_by_email(email).await
    }

    pub async fn create_user(&self, user: &User) -> Result<User, Error> {
        let created = self.users.create(user).await?;
        Ok(created)
    }

    pub async fn update_user(&self, user: &User) -> Result<User, Error> {
        let updated = self.users.update(user).await?;
        Ok(updated)
    }

    pub async fn delete_user(&self, id: Uuid) -> Result<(), Error> {
        self.users.delete(id).await?;
        Ok(())
    }

    // Account methods
    pub async fn get_accounts(&self, user_id: Uuid) -> Result<Vec<Account>, Error> {
        self.accounts.find_by_user_id(user_id).await
    }

    pub async fn connect_account(&self, account: &Account) -> Result<Account, Error> {
        let created = self.accounts.create(account).await?;
        
        self.event_bus.publish(Event::AccountConnected {
            account_id: created.id,
            user_id: created.user_id,
            connector_name: created.connector_name.clone(),
        }).await.map_err(|e| Box::new(std::io::Error::new(std::io::ErrorKind::Other, e)) as Error)?;

        Ok(created)
    }

    pub async fn disconnect_account(&self, id: Uuid) -> Result<(), Error> {
        if let Some(account) = self.accounts.find_by_id(id).await? {
            self.accounts.delete(id).await?;
            
            self.event_bus.publish(Event::AccountDisconnected {
                account_id: id,
                user_id: account.user_id,
                connector_name: account.connector_name,
            }).await.map_err(|e| Box::new(std::io::Error::new(std::io::ErrorKind::Other, e)) as Error)?;
        }
        Ok(())
    }

    // Contact methods
    pub async fn get_contacts(&self, user_id: Uuid) -> Result<Vec<Contact>, Error> {
        self.contacts.find_by_user_id(user_id).await
    }

    pub async fn create_contact(&self, contact: &Contact) -> Result<Contact, Error> {
        let created = self.contacts.create(contact).await?;
        
        self.event_bus.publish(Event::ContactCreated {
            contact_id: created.id,
            user_id: created.user_id,
        }).await.map_err(|e| Box::new(std::io::Error::new(std::io::ErrorKind::Other, e)) as Error)?;

        Ok(created)
    }

    // Conversation methods
    pub async fn get_conversations(&self, user_id: Uuid) -> Result<Vec<Conversation>, Error> {
        self.conversations.find_by_user_id(user_id).await
    }

    pub async fn list_all_conversations(&self) -> Result<Vec<Conversation>, Error> {
        self.conversations.list_all().await
    }

    pub async fn find_conversation_by_title_prefix(&self, prefix: &str) -> Result<Option<Conversation>, Error> {
        self.conversations.find_by_title_prefix(prefix).await
    }

    pub async fn get_conversation(&self, id: Uuid) -> Result<Option<Conversation>, Error> {
        self.conversations.find_by_id(id).await
    }

    pub async fn update_conversation(&self, conversation: &Conversation) -> Result<Conversation, Error> {
        self.conversations.update(conversation).await
    }

    pub async fn create_conversation(&self, conversation: &Conversation) -> Result<Conversation, Error> {
        let created = self.conversations.create(conversation).await?;
        
        self.event_bus.publish(Event::ConversationCreated {
            conversation_id: created.id,
            user_id: created.user_id,
        }).await.map_err(|e| Box::new(std::io::Error::new(std::io::ErrorKind::Other, e)) as Error)?;

        Ok(created)
    }

    // Message methods
    pub async fn get_messages(&self, conversation_id: Uuid) -> Result<Vec<Message>, Error> {
        self.messages.find_by_conversation(conversation_id).await
    }

    pub async fn send_message(&self, message: &Message) -> Result<Message, Error> {
        let created = self.messages.create(message).await?;
        
        self.event_bus.publish(Event::MessageSent {
            message_id: created.id,
            conversation_id: created.conversation_id,
            connector_id: created.connector_id.clone(),
        }).await.map_err(|e| Box::new(std::io::Error::new(std::io::ErrorKind::Other, e)) as Error)?;

        Ok(created)
    }

    pub async fn receive_message(&self, message: &Message) -> Result<Message, Error> {
        let created = self.messages.create(message).await?;
        
        self.event_bus.publish(Event::MessageReceived {
            message_id: created.id,
            conversation_id: created.conversation_id,
            sender_id: created.sender_id,
            connector_id: created.connector_id.clone(),
        }).await.map_err(|e| Box::new(std::io::Error::new(std::io::ErrorKind::Other, e)) as Error)?;

        Ok(created)
    }

    // Event bus access
    pub fn event_bus(&self) -> &EventBus {
        &self.event_bus
    }
}
