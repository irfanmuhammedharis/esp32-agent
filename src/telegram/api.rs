//! Telegram Bot HTTP API client — blocking, synchronous.

use crate::telegram::types::{GetUpdatesResponse, Message, Update};
use embedded_svc::http::client::Client as HttpClient;
use embedded_svc::http::Method;
use esp_idf_svc::http::client::{Configuration as HttpConfig, EspHttpConnection};
use heapless::String;
use log::{error, info};

const BASE_URL: &str = "https://api.telegram.org/bot";

pub struct TelegramApi {
    token: String<128>,
    offset: i64,
}

impl TelegramApi {
    pub fn new(token: &str) -> anyhow::Result<Self> {
        let mut t = String::<128>::new();
        t.push_str(token).map_err(|_| anyhow::anyhow!("token too long"))?;
        Ok(Self { token: t, offset: 0 })
    }

    /// Blocking long-poll. Returns up to 8 messages.
    pub fn get_updates(&mut self) -> anyhow::Result<heapless::Vec<(i64, String<256>), 8>> {
        let url = format!("{}{}/getUpdates?limit=8&timeout=30&offset={}", BASE_URL, self.token.as_str(), self.offset);

        let mut client = HttpClient::wrap(
            EspHttpConnection::new(&HttpConfig {
                use_global_ca_store: true,
                crt_bundle_attach: Some(esp_idf_sys::esp_crt_bundle_attach),
                ..Default::default()
            })?,
        );

        let mut request = client.request(Method::Get, &url, &[])?;
        let mut response = request.submit()?;

        let status = response.status();
        if status != 200 {
            return Err(anyhow::anyhow!("Telegram HTTP {}", status));
        }

        let mut buf = [0u8; 4096];
        let mut read = 0usize;
        loop {
            let n = response.read(&mut buf[read..])?;
            if n == 0 { break; }
            read += n;
            if read >= buf.len() { break; }
        }

        let text = core::str::from_utf8(&buf[..read])?;
        let parsed: GetUpdatesResponse = serde_json::from_str(text)
            .map_err(|e| {
                error!("[TG] JSON parse error: {:?}", e);
                anyhow::anyhow!("JSON parse: {:?}", e)
            })?;

        let mut out = heapless::Vec::<(i64, String<256>), 8>::new();
        if let Some(updates) = parsed.result {
            for up in updates {
                if up.update_id >= self.offset {
                    self.offset = up.update_id + 1;
                }
                if let Some(msg) = up.message {
                    if let Some(txt) = msg.text {
                        let mut text_buf = String::<256>::new();
                        if text_buf.push_str(&txt).is_ok() {
                            let _ = out.push((msg.chat.id, text_buf));
                        }
                    }
                }
            }
        }

        Ok(out)
    }

    /// Send a text message to a chat.
    pub fn send_message(&mut self, chat_id: i64, text: &str) -> anyhow::Result<()> {
        let url = format!("{}{}/sendMessage", BASE_URL, self.token.as_str());

        let body = serde_json::json!({
            "chat_id": chat_id,
            "text": text,
        });
        let body_str = body.to_string();

        let mut client = HttpClient::wrap(
            EspHttpConnection::new(&HttpConfig {
                use_global_ca_store: true,
                crt_bundle_attach: Some(esp_idf_sys::esp_crt_bundle_attach),
                ..Default::default()
            })?,
        );

        let headers = [
            ("Content-Type", "application/json"),
        ];

        let mut request = client.request(Method::Post, &url, &headers)?;
        request.write(body_str.as_bytes())?;
        let mut response = request.submit()?;

        let status = response.status();
        if status != 200 {
            error!("[TG] send_message HTTP {}", status);
            return Err(anyhow::anyhow!("send_message HTTP {}", status));
        }

        // Drain response
        let mut discard = [0u8; 256];
        loop {
            let n = response.read(&mut discard)?;
            if n == 0 { break; }
        }

        Ok(())
    }
}
