use axum::{
    Json,
    extract::State,
};
use serde::{Deserialize, Serialize};

use crate::api::{ApiResponse, AppState};

const SIDECAR_URL: &str = "http://127.0.0.1:3001";

#[derive(Deserialize)]
pub struct PairCodeRequest {
    pub phone_number: String,
}

#[derive(Serialize)]
pub struct WhatsAppStatusResponse {
    pub status: String,
    pub message: String,
    pub user: Option<serde_json::Value>,
}

#[derive(Serialize)]
pub struct QRResponse {
    pub qr: Option<String>,
    pub status: String,
}

pub async fn whatsapp_start(
    State(_state): State<AppState>,
) -> Json<ApiResponse<String>> {
    match reqwest::Client::new()
        .post(format!("{}/start", SIDECAR_URL))
        .send()
        .await
    {
        Ok(r) => {
            let d: serde_json::Value = r.json().await.unwrap_or_default();
            if d.get("ok").and_then(|v| v.as_bool()).unwrap_or(false) {
                Json(ApiResponse::success("WhatsApp sidecar started".to_string()))
            } else {
                let err = d.get("error").and_then(|v| v.as_str()).unwrap_or("unknown error");
                Json(ApiResponse::error(err))
            }
        }
        Err(e) => Json(ApiResponse::error(&format!("Sidecar unreachable: {}", e))),
    }
}

pub async fn whatsapp_qr(
    State(_state): State<AppState>,
) -> Json<ApiResponse<QRResponse>> {
    match reqwest::Client::new()
        .get(format!("{}/qr", SIDECAR_URL))
        .send()
        .await
    {
        Ok(r) => {
            let d: serde_json::Value = r.json().await.unwrap_or_default();
            let qr = d.get("qr").and_then(|v| v.as_str()).map(|s| s.to_string());
            let status = d.get("status").and_then(|v| v.as_str()).unwrap_or("unknown").to_string();
            Json(ApiResponse::success(QRResponse { qr, status }))
        }
        Err(e) => Json(ApiResponse::error(&format!("Sidecar unreachable: {}", e))),
    }
}

pub async fn whatsapp_pair_code(
    State(_state): State<AppState>,
    Json(req): Json<PairCodeRequest>,
) -> Json<ApiResponse<String>> {
    match reqwest::Client::new()
        .post(format!("{}/pair-code", SIDECAR_URL))
        .json(&serde_json::json!({ "phoneNumber": req.phone_number }))
        .send()
        .await
    {
        Ok(r) => {
            let d: serde_json::Value = r.json().await.unwrap_or_default();
            if d.get("ok").and_then(|v| v.as_bool()).unwrap_or(false) {
                let code = d.get("code").and_then(|v| v.as_str()).unwrap_or("").to_string();
                Json(ApiResponse::success(code))
            } else {
                let err = d.get("error").and_then(|v| v.as_str()).unwrap_or("unknown error");
                Json(ApiResponse::error(err))
            }
        }
        Err(e) => Json(ApiResponse::error(&format!("Sidecar unreachable: {}", e))),
    }
}

pub async fn whatsapp_status(
    State(_state): State<AppState>,
) -> Json<ApiResponse<WhatsAppStatusResponse>> {
    match reqwest::Client::new()
        .get(format!("{}/status", SIDECAR_URL))
        .send()
        .await
    {
        Ok(r) => {
            let d: serde_json::Value = r.json().await.unwrap_or_default();
            let status = d.get("status").and_then(|v| v.as_str()).unwrap_or("disconnected").to_string();
            let message = d.get("message").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let user = d.get("user").cloned();
            Json(ApiResponse::success(WhatsAppStatusResponse { status, message, user }))
        }
        Err(e) => Json(ApiResponse::error(&format!("Sidecar unreachable: {}", e))),
    }
}

pub async fn whatsapp_disconnect(
    State(_state): State<AppState>,
) -> Json<ApiResponse<String>> {
    match reqwest::Client::new()
        .post(format!("{}/disconnect", SIDECAR_URL))
        .send()
        .await
    {
        Ok(r) => {
            let d: serde_json::Value = r.json().await.unwrap_or_default();
            if d.get("ok").and_then(|v| v.as_bool()).unwrap_or(false) {
                Json(ApiResponse::success("Disconnected".to_string()))
            } else {
                let err = d.get("error").and_then(|v| v.as_str()).unwrap_or("unknown error");
                Json(ApiResponse::error(err))
            }
        }
        Err(e) => Json(ApiResponse::error(&format!("Sidecar unreachable: {}", e))),
    }
}
