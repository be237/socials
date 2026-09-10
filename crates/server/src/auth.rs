use axum::{
    Json,
    extract::State,
};
use serde::{Deserialize, Serialize};

use socials_telegram_user::AuthState;
use crate::api::{ApiResponse, AppState};

#[derive(Deserialize)]
pub struct RequestCodeRequest {
    pub phone: String,
}

#[derive(Deserialize)]
pub struct SignInRequest {
    pub code: String,
}

#[derive(Serialize)]
pub struct AuthStatusResponse {
    pub state: AuthState,
    pub has_session: bool,
}

pub async fn telegram_request_code(
    State(state): State<AppState>,
    Json(req): Json<RequestCodeRequest>,
) -> Json<ApiResponse<String>> {
    let mut client = state.auth.telegram.write().await;

    match client.connect().await {
        Ok(_) => {}
        Err(e) => return Json(ApiResponse::error(&format!("Connection error: {}", e))),
    }

    match client.request_code(&req.phone).await {
        Ok(()) => Json(ApiResponse::success("Code sent".to_string())),
        Err(e) => Json(ApiResponse::error(&format!("Failed to send code: {}", e))),
    }
}

pub async fn telegram_sign_in(
    State(state): State<AppState>,
    Json(req): Json<SignInRequest>,
) -> Json<ApiResponse<String>> {
    let mut client = state.auth.telegram.write().await;

    match client.sign_in(&req.code).await {
        Ok(msg) => {
            Json(ApiResponse::success(msg))
        }
        Err(e) => Json(ApiResponse::error(&e.to_string())),
    }
}

pub async fn telegram_status(
    State(state): State<AppState>,
) -> Json<ApiResponse<AuthStatusResponse>> {
    let client = state.auth.telegram.read().await;
    let auth_state = client.auth_state.read().await.clone();
    let has_session = client.is_authorized().await.unwrap_or(false);

    Json(ApiResponse::success(AuthStatusResponse {
        state: auth_state,
        has_session,
    }))
}

pub async fn telegram_disconnect(
    State(state): State<AppState>,
) -> Json<ApiResponse<String>> {
    let mut client = state.auth.telegram.write().await;

    match client.disconnect().await {
        Ok(()) => Json(ApiResponse::success("Disconnected".to_string())),
        Err(e) => Json(ApiResponse::error(&format!("Disconnect failed: {}", e))),
    }
}

pub async fn get_accounts(
    State(state): State<AppState>,
) -> Json<ApiResponse<Vec<AccountInfo>>> {
    let mut accounts = Vec::new();

    // Check Telegram
    {
        let client = state.auth.telegram.read().await;
        let auth_state = client.auth_state.read().await.clone();
        if let AuthState::Connected { user_id, username } = auth_state {
            accounts.push(AccountInfo {
                connector: "telegram".to_string(),
                display_name: username,
                platform_account_id: user_id.to_string(),
                is_connected: true,
            });
        }
    }

    // Check WhatsApp via sidecar
    if let Ok(r) = reqwest::Client::new()
        .get("http://127.0.0.1:3001/status")
        .send()
        .await
    {
        if let Ok(d) = r.json::<serde_json::Value>().await {
            if d.get("status").and_then(|v| v.as_str()) == Some("connected") {
                let user_name = d.get("user")
                    .and_then(|u| u.get("name"))
                    .and_then(|n| n.as_str())
                    .unwrap_or("WhatsApp")
                    .to_string();
                let user_id = d.get("user")
                    .and_then(|u| u.get("id"))
                    .and_then(|n| n.as_str())
                    .unwrap_or("")
                    .to_string();
                accounts.push(AccountInfo {
                    connector: "whatsapp".to_string(),
                    display_name: user_name,
                    platform_account_id: user_id,
                    is_connected: true,
                });
            }
        }
    }

    Json(ApiResponse::success(accounts))
}

#[derive(Serialize)]
pub struct AccountInfo {
    pub connector: String,
    pub display_name: String,
    pub platform_account_id: String,
    pub is_connected: bool,
}
