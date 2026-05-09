//! Minimal Telegram Bot API types.

use heapless::Vec;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Update {
    #[serde(rename = "update_id")]
    pub update_id: i64,
    pub message: Option<Message>,
}

#[derive(Debug, Deserialize)]
pub struct Message {
    #[serde(rename = "message_id")]
    pub message_id: i64,
    pub from: Option<User>,
    pub chat: Chat,
    pub text: Option<std::string::String>,
}

#[derive(Debug, Deserialize)]
pub struct User {
    pub id: i64,
    #[serde(rename = "first_name")]
    pub first_name: Option<std::string::String>,
}

#[derive(Debug, Deserialize)]
pub struct Chat {
    pub id: i64,
}

/// Wrapper for the `getUpdates` response.
#[derive(Debug, Deserialize)]
pub struct GetUpdatesResponse {
    pub ok: bool,
    pub result: Option<Vec<Update, 16>>,
}

/// Wrapper for generic API responses.
#[derive(Debug, Deserialize)]
pub struct ApiResponse<T> {
    pub ok: bool,
    pub result: Option<T>,
}
