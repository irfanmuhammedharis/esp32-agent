//! FreeRTOS-oriented types and inter-task communication primitives.
//!
//! On ESP-IDF, `std::thread` maps to FreeRTOS tasks and
//! `std::sync::mpsc::sync_channel` maps to FreeRTOS queues + mutexes.

use heapless::String;

/// Queue depth for pending projects.
pub const PLAN_QUEUE_DEPTH: usize = 2;

// ------------------------------------------------------------------
// Command — local serial commands (never touch hardware directly)
// ------------------------------------------------------------------

/// A parsed user command from Serial.
/// NOTE: All hardware-related instructions are treated as natural-language
/// strings and forwarded to the LLM for absolute control.
#[derive(Debug, Clone)]
pub enum Command {
    /// Natural-language instruction to be sent to the LLM.
    LlmTask {
        text: String<256>,
    },
    /// Print GPIO capability table (read-only, no hardware change).
    GpioList,
    /// Configuration update (local-only, no hardware change).
    ConfigSet { key: String<32>, value: String<128> },
    /// Software reset.
    Reset,
    /// Print help.
    Help,
    /// Unknown / no-op.
    Noop,
}
