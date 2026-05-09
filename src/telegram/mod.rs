//! Telegram bot polling task.

use crate::config::Config;
use crate::llm::deepseek::DeepSeekClient;
use crate::llm::LlmClient;
use crate::llm::plan::Project;
use crate::telegram::api::TelegramApi;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use heapless::String;
use log::{error, info, warn};
use std::sync::mpsc::SyncSender;
use std::time::{Duration, Instant};

pub mod api;
pub mod types;

/// Maximum requests per minute to the LLM API.
const MAX_REQUESTS_PER_MINUTE: u8 = 10;

/// Entry point for the Telegram task.
pub fn run(
    cfg: Config,
    tx: SyncSender<Project>,
    _nvs: EspDefaultNvsPartition,
) {
    info!("[TG] Task started.");

    let mut api = match TelegramApi::new(cfg.telegram_token.as_str()) {
        Ok(a) => a,
        Err(e) => {
            error!("[TG] Failed to create API client: {:?}", e);
            return;
        }
    };

    let llm = match DeepSeekClient::new(cfg.deepseek_key.as_str()) {
        Ok(c) => c,
        Err(e) => {
            error!("[TG] Failed to create LLM client: {:?}", e);
            return;
        }
    };

    let mut requests: u8 = 0;
    let mut window_start = Instant::now();

    loop {
        // Simple token-bucket rate limiter
        if window_start.elapsed() >= Duration::from_secs(60) {
            requests = 0;
            window_start = Instant::now();
        }

        match api.get_updates() {
            Ok(updates) => {
                for (chat_id, text) in updates {
                    info!("[TG] Message from {}: {}", chat_id, text);

                    // Whitelist check
                    if cfg.allowed_user_id != 0 && chat_id != cfg.allowed_user_id {
                        warn!("[TG] Rejected message from {}", chat_id);
                        continue;
                    }

                    if !requests_consume(&mut requests) {
                        let _ = api.send_message(chat_id, "Rate limit reached. Please wait.");
                        continue;
                    }

                    match llm.complete(text.as_str()) {
                        Ok(project) => {
                            // Send wiring instructions + description back to user
                            let mut reply = String::<512>::new();
                            let _ = reply.push_str(&project.description);
                            let _ = reply.push_str("\n\n🔌 WIRING:\n");
                            let _ = reply.push_str(&project.wiring_instructions);
                            let _ = reply.push_str("\n\n✅ Program loaded and running on device.");
                            let _ = api.send_message(chat_id, reply.as_str());
                            let _ = tx.send(project);
                        }
                        Err(e) => {
                            error!("[TG] LLM error: {:?}", e);
                            let mut msg = String::<128>::new();
                            let _ = msg.push_str("LLM error: ");
                            let _ = msg.push_str(&format!("{:?}", e));
                            let _ = api.send_message(chat_id, msg.as_str());
                        }
                    }
                }
            }
            Err(e) => {
                error!("[TG] get_updates error: {:?}", e);
                std::thread::sleep(Duration::from_secs(5));
            }
        }
    }
}

fn requests_consume(counter: &mut u8) -> bool {
    if *counter >= MAX_REQUESTS_PER_MINUTE {
        false
    } else {
        *counter += 1;
        true
    }
}
