use axum::{
    Router,
    routing::{get, post},
    Json,
    extract::{State, Path},
};
use tower_http::cors::{CorsLayer, Any};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::Utc;
use std::sync::Arc;
use tokio::sync::RwLock;

use socials_core::entities::conversation::{Conversation, ConversationType};
use socials_core::entities::message::{Message, MessageType, MessageStatus};
use socials_core::entities::user::User;
use socials_core::services::CoreService;
use crate::telegram::TelegramBot;
use crate::auth;
use crate::whatsapp;
use socials_telegram_user::TelegramUserClient;

#[derive(Clone)]
pub struct AuthState {
    pub telegram: Arc<RwLock<TelegramUserClient>>,
}

#[derive(Clone)]
pub struct AppState {
    pub core: Arc<CoreService>,
    pub telegram: Arc<RwLock<Option<TelegramBot>>>,
    pub auth: AuthState,
}

#[derive(Serialize, Deserialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self { success: true, data: Some(data), error: None }
    }
    pub fn error(msg: &str) -> Self {
        Self { success: false, data: None, error: Some(msg.to_string()) }
    }
}

#[derive(Deserialize)]
pub struct CreateConversationRequest {
    pub title: String,
    pub conversation_type: Option<String>,
}

#[derive(Deserialize)]
pub struct SendMessageRequest {
    pub conversation_id: Uuid,
    pub content: String,
    pub sender_name: Option<String>,
}

pub async fn seed_demo_data(core: &CoreService) {
    if let Ok(convos) = core.list_all_conversations().await {
        if !convos.is_empty() {
            return;
        }
    }

    let user_id = Uuid::new_v4();
    let now = Utc::now();
    let demo_user = User {
        id: user_id,
        username: "user".to_string(),
        email: "user@socials.local".to_string(),
        created_at: now,
        updated_at: now,
    };
    let _ = core.create_user(&demo_user).await;

    let demo_data = vec![
        ("Telegram - Alice", vec![
            ("Alice", "Hey, tu es la ?"),
            ("Vous", "Oui ! Quoi de neuf ?"),
            ("Alice", "On se voit ce weekend ?"),
            ("Vous", "Bonne idee ! Samedi aprem ?"),
            ("Alice", "Parfait, au cafe a 15h"),
        ]),
        ("Discord - Dev Team", vec![
            ("Marc", "Le build est passe !"),
            ("Sophie", "Super, je merge"),
            ("Vous", "Je review la PR"),
            ("Marc", "Merci"),
        ]),
        ("Gmail - Newsletter", vec![
            ("Newsletter", "Votre resume tech de la semaine"),
        ]),
        ("WhatsApp - Famille", vec![
            ("Maman", "N'oublie pas le diner dimanche"),
            ("Vous", "J'y serai !"),
            ("Papa", "A 19h pile"),
            ("Vous", "OK"),
        ]),
    ];

    let mut offset = 0i64;
    for (title, msgs) in demo_data {
        let conv = Conversation {
            id: Uuid::new_v4(),
            user_id,
            conversation_type: ConversationType::Private,
            title: title.to_string(),
            created_at: now,
            updated_at: now,
            last_message_at: Some(now),
        };

        if let Ok(created_conv) = core.create_conversation(&conv).await {
            for (sender, content) in msgs {
                offset += 600;
                let msg_time = Utc::now() - chrono::Duration::seconds(offset);
                let msg = Message {
                    id: Uuid::new_v4(),
                    conversation_id: created_conv.id,
                    sender_id: Uuid::new_v4(),
                    content: content.to_string(),
                    message_type: MessageType::Text,
                    status: MessageStatus::Sent,
                    created_at: msg_time,
                    edited_at: None,
                    reply_to: None,
                    connector_id: sender.to_string(),
                    platform_message_id: Uuid::new_v4().to_string(),
                };
                let _ = core.receive_message(&msg).await;
            }
        }
    }
}

pub fn create_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/api/conversations", get(get_conversations).post(create_conversation))
        .route("/api/conversations/:id", get(get_conversation))
        .route("/api/conversations/:id/messages", get(get_messages))
        .route("/api/messages", post(send_message))
        .route("/api/health", get(health_check))
        .route("/api/accounts", get(auth::get_accounts))
        .route("/api/auth/telegram/request-code", post(auth::telegram_request_code))
        .route("/api/auth/telegram/sign-in", post(auth::telegram_sign_in))
        .route("/api/auth/telegram/status", get(auth::telegram_status))
        .route("/api/auth/telegram/disconnect", post(auth::telegram_disconnect))
        .route("/api/auth/whatsapp/start", post(whatsapp::whatsapp_start))
        .route("/api/auth/whatsapp/qr", get(whatsapp::whatsapp_qr))
        .route("/api/auth/whatsapp/pair-code", post(whatsapp::whatsapp_pair_code))
        .route("/api/auth/whatsapp/status", get(whatsapp::whatsapp_status))
        .route("/api/auth/whatsapp/disconnect", post(whatsapp::whatsapp_disconnect))
        .layer(cors)
        .with_state(state)
}

async fn health_check() -> Json<ApiResponse<&'static str>> {
    Json(ApiResponse::success("ok"))
}

async fn get_conversations(
    State(state): State<AppState>,
) -> Json<ApiResponse<Vec<Conversation>>> {
    match state.core.list_all_conversations().await {
        Ok(convos) => Json(ApiResponse::success(convos)),
        Err(e) => Json(ApiResponse::error(&e.to_string())),
    }
}

async fn create_conversation(
    State(state): State<AppState>,
    Json(request): Json<CreateConversationRequest>,
) -> Json<ApiResponse<Conversation>> {
    let conv_type = match request.conversation_type.as_deref() {
        Some("group") => ConversationType::Group,
        Some("channel") => ConversationType::Channel,
        Some("email") => ConversationType::Email,
        _ => ConversationType::Private,
    };

    let now = Utc::now();
    let conversation = Conversation {
        id: Uuid::new_v4(),
        user_id: Uuid::new_v4(),
        conversation_type: conv_type,
        title: request.title,
        created_at: now,
        updated_at: now,
        last_message_at: None,
    };

    match state.core.create_conversation(&conversation).await {
        Ok(created) => Json(ApiResponse::success(created)),
        Err(e) => Json(ApiResponse::error(&e.to_string())),
    }
}

async fn get_conversation(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Json<ApiResponse<Option<Conversation>>> {
    match state.core.get_conversation(id).await {
        Ok(conv) => Json(ApiResponse::success(conv)),
        Err(e) => Json(ApiResponse::error(&e.to_string())),
    }
}

async fn get_messages(
    State(state): State<AppState>,
    Path(conversation_id): Path<Uuid>,
) -> Json<ApiResponse<Vec<Message>>> {
    match state.core.get_messages(conversation_id).await {
        Ok(msgs) => Json(ApiResponse::success(msgs)),
        Err(e) => Json(ApiResponse::error(&e.to_string())),
    }
}

async fn send_message(
    State(state): State<AppState>,
    Json(request): Json<SendMessageRequest>,
) -> Json<ApiResponse<Message>> {
    let sender_name = request.sender_name.unwrap_or_else(|| "Vous".to_string());
    let now = Utc::now();
    let message = Message {
        id: Uuid::new_v4(),
        conversation_id: request.conversation_id,
        sender_id: Uuid::new_v4(),
        content: request.content.clone(),
        message_type: MessageType::Text,
        status: MessageStatus::Sent,
        created_at: now,
        edited_at: None,
        reply_to: None,
        connector_id: sender_name,
        platform_message_id: Uuid::new_v4().to_string(),
    };

    match state.core.send_message(&message).await {
        Ok(saved_msg) => {
            // Update last_message_at on conversation
            if let Ok(Some(mut conv)) = state.core.get_conversation(request.conversation_id).await {
                conv.last_message_at = Some(now);
                conv.updated_at = now;
                let _ = state.core.update_conversation(&conv).await;

                // Send to Telegram if this conversation title maps to a Telegram chat
                if let Some(chat_id) = TelegramBot::extract_chat_id(&conv.title) {
                    if let Some(bot) = state.telegram.read().await.as_ref() {
                        if let Err(e) = bot.send_message(chat_id, &request.content).await {
                            eprintln!("Failed to send to Telegram: {}", e);
                        }
                    }
                }
            }

            Json(ApiResponse::success(saved_msg))
        }
        Err(e) => Json(ApiResponse::error(&e.to_string())),
    }
}
