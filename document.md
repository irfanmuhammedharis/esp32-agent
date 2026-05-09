# ESP32-Agent
## AI-Driven GPIO Controller for ESP32
### Powered by Rust · Telegram Bot · DeepSeek LLM

> **Version 1.0 · May 2026 · Marinode-AI**  
> Technical Architecture & Developer Guide

---

## Table of Contents

1. [Project Overview](#1-project-overview)
2. [System Requirements](#2-system-requirements)
3. [Repository Layout](#3-repository-layout)
4. [Toolchain Setup](#4-toolchain-setup)
5. [Configuration](#5-configuration)
6. [Core Implementation](#6-core-implementation)
7. [End-to-End Message Flow](#7-end-to-end-message-flow)
8. [Serial Monitor Interface](#8-serial-monitor-interface)
9. [Memory Management Strategy](#9-memory-management-strategy)
10. [Security Considerations](#10-security-considerations)
11. [Usage Examples](#11-usage-examples)
12. [Build, Flash & Monitor](#12-build-flash--monitor)
13. [Extending ESP32-Agent](#13-extending-esp32-agent)
14. [Troubleshooting](#14-troubleshooting)
15. [Roadmap](#15-roadmap)
16. [Glossary](#16-glossary)
17. [References & Further Reading](#17-references--further-reading)

---

## 1. Project Overview

ESP32-Agent is an open-source, AI-driven GPIO control framework for the ESP32 microcontroller, implemented entirely in Rust. Inspired by the OpenClaw project, it enables natural-language task delegation via Telegram — the user sends a human-readable instruction, and the system autonomously reasons over pin capabilities using a DeepSeek LLM, generates the required control logic, and executes it in real-time.

Unlike conventional firmware that requires hard-coded pin mappings, ESP32-Agent is **self-programming**: every instruction is semantically parsed, the LLM maps intent to physical GPIO, and the resulting runtime action is streamed back to the user over Telegram or USB Serial — with zero manual recompilation.

### 1.1 Design Pillars

- **Zero-recompile workflow** — send a task, get execution; no IDE needed at runtime
- **Memory efficiency** — Rust's ownership model, `no_std` where possible, static allocation
- **Secure & auditable** — all LLM-generated plans are printed before execution
- **Dual interface** — Telegram bot for remote control + Serial monitor for local debug
- **Extensible** — plug-in new LLM backends or communication transports

### 1.2 High-Level Architecture

The system is composed of five cooperating layers:

| Layer | Component | Responsibility |
|-------|-----------|----------------|
| Transport | Telegram Bot / UART Serial | Receive user instructions, stream responses |
| Intelligence | DeepSeek API Client | Parse intent, generate pin assignment plans |
| Runtime | Task Executor | Apply GPIO state from LLM plan |
| Hardware | ESP32 GPIO Driver (esp-idf-hal) | Safe pin access with ownership semantics |
| Persistence | NVS Flash Store | Save Wi-Fi credentials, bot token, API key |

---

## 2. System Requirements

### 2.1 Hardware

| Parameter | Specification |
|-----------|--------------|
| MCU | ESP32 or ESP32-S3 (dual-core Xtensa, 240 MHz, 520 KB SRAM, 4 MB+ Flash) |
| Flash | Minimum 4 MB; 8 MB recommended for OTA support |
| PSRAM | Optional — 8 MB PSRAM improves JSON buffer headroom |
| USB | CP2102/CH340 USB-UART or native USB-OTG (ESP32-S3) |
| Network | 2.4 GHz Wi-Fi (802.11 b/g/n) — required for Telegram and LLM API |
| Power | 5V/500 mA via USB; 3.3V logic level on all GPIO |

### 2.2 Software & Toolchain

| Item | Value |
|------|-------|
| Language | Rust (stable 1.78+) |
| Build system | Cargo with esp-idf-sys build integration |
| ESP-IDF | v5.1.x (set via `IDF_PATH` or managed by `esp-idf-sys`) |
| Target triple | `xtensa-esp32-espidf` or `xtensa-esp32s3-espidf` |
| Rust toolchain | `esp` (Espressif fork with Xtensa LLVM backend) |
| Flashing tool | `cargo-espflash` v3.x |
| Monitor tool | `espflash monitor` or `cargo-espflash monitor` |
| OS (dev host) | Linux / macOS / Windows (WSL2 strongly recommended) |

### 2.3 External Services

| Service | Detail |
|---------|--------|
| Telegram Bot | Create via @BotFather; obtain HTTP API token |
| DeepSeek API | Register at platform.deepseek.com; generate API key |
| Wi-Fi network | 2.4 GHz SSID + password for ESP32 connection |

---

## 3. Repository Layout

```
esp32-agent/
├── Cargo.toml                  # workspace root
├── sdkconfig.defaults          # ESP-IDF menuconfig baseline
├── build.rs                    # esp-idf-sys binding generation
├── .cargo/
│   └── config.toml             # target, runner, linker flags
├── src/
│   ├── main.rs                 # entry point, task spawning
│   ├── config.rs               # NVS-backed configuration store
│   ├── wifi.rs                 # Wi-Fi station mode driver
│   ├── telegram/
│   │   ├── mod.rs              # bot polling loop
│   │   ├── api.rs              # Telegram HTTP API client
│   │   └── types.rs            # Telegram message structs
│   ├── llm/
│   │   ├── mod.rs              # LLM abstraction trait
│   │   ├── deepseek.rs         # DeepSeek chat/completions client
│   │   └── plan.rs             # Plan struct + JSON deserialiser
│   ├── gpio/
│   │   ├── mod.rs              # GPIO registry, safe handle store
│   │   ├── executor.rs         # Applies LLM plan to hardware
│   │   └── registry.rs         # Static pin capability table
│   ├── serial.rs               # USB-UART loopback + command parser
│   └── utils.rs                # Stack-allocated string helpers
└── docs/                       # This document and diagrams
```

---

## 4. Toolchain Setup

### 4.1 Install Rust with Espressif Support

```bash
# Install rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install the Espressif Rust toolchain via espup
cargo install espup
espup install            # downloads xtensa-enabled LLVM + std library

# Source the environment (add to ~/.bashrc for persistence)
. $HOME/export-esp.sh
```

### 4.2 Install ESP-IDF v5.1

```bash
git clone --recursive https://github.com/espressif/esp-idf.git
cd esp-idf && git checkout v5.1.3
./install.sh esp32
. ./export.sh
```

### 4.3 Install cargo-espflash

```bash
cargo install cargo-espflash

# Verify
cargo espflash --version
```

### 4.4 Project Bootstrap

```bash
git clone https://github.com/your-org/esp32-agent.git
cd esp32-agent

# Copy env template and fill credentials
cp .env.example .env
# Edit .env → set WIFI_SSID, WIFI_PASS, TELEGRAM_TOKEN, DEEPSEEK_API_KEY
```

---

## 5. Configuration

### 5.1 Cargo.toml — Key Dependencies

```toml
[package]
name    = "esp32-agent"
version = "1.0.0"
edition = "2021"

[dependencies]
esp-idf-sys    = { version = "0.35", features = ["binstart"] }
esp-idf-hal    = "0.44"
esp-idf-svc    = "0.48"         # WiFi, HTTP, NVS
embedded-hal   = "1.0"
embedded-svc   = "0.27"
serde          = { version = "1", default-features = false, features = ["derive"] }
serde_json     = { version = "1", default-features = false, features = ["alloc"] }
heapless       = "0.8"          # stack-allocated collections
log            = "0.4"
anyhow         = { version = "1", default-features = false }

[profile.release]
opt-level     = "z"             # minimise binary size
lto           = true
codegen-units = 1
```

### 5.2 .cargo/config.toml

```toml
[build]
target = "xtensa-esp32-espidf"

[target.xtensa-esp32-espidf]
linker  = "ldproxy"
runner  = "cargo-espflash flash --monitor"

[env]
ESP_IDF_VERSION = "v5.1.3"
MCU             = "esp32"
```

---

## 6. Core Implementation

### 6.1 main.rs — Entry Point & Task Orchestration

ESP-IDF uses FreeRTOS under the hood. Rust wraps this via `esp-idf-svc`. The entry point spawns three independent tasks on the FreeRTOS scheduler:

- **`telegram_task`** — polls Telegram `getUpdates`, dispatches messages
- **`serial_task`** — reads UART0, mirrors commands through the same pipeline
- **`gpio_task`** — receives `PinPlan` structs from the LLM pipeline via a queue

```rust
// src/main.rs
#![no_std]
#![no_main]

extern crate esp_idf_sys;          // must link first

use esp_idf_hal::prelude::*;
use esp_idf_svc::{eventloop::EspSystemEventLoop, nvs::EspDefaultNvsPartition};

#[no_mangle]
fn app_main() {
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();
    let sysloop     = EspSystemEventLoop::take().unwrap();
    let nvs         = EspDefaultNvsPartition::take().unwrap();

    // Load persisted config (Wi-Fi creds, tokens, API key)
    let cfg = config::Config::load(&nvs).expect("NVS config missing");

    // Bring up Wi-Fi station
    let _wifi = wifi::connect(&cfg, &sysloop, &nvs, peripherals.modem)
        .expect("Wi-Fi failed");

    // Shared command queue (capacity = 4 plan structs)
    static PLAN_QUEUE: heapless::spsc::Queue<gpio::PinPlan, 4> =
        heapless::spsc::Queue::new();
    let (producer, consumer) = PLAN_QUEUE.split();

    // Spawn tasks
    std::thread::Builder::new().stack_size(8192)
        .spawn(move || telegram::run(cfg, producer)).unwrap();

    std::thread::Builder::new().stack_size(4096)
        .spawn(move || serial::run()).unwrap();

    gpio::executor::run(consumer, peripherals.pins);
}
```

### 6.2 config.rs — NVS-Backed Secure Storage

Credentials are stored in ESP32 Non-Volatile Storage (NVS), a key-value store that survives reboots and OTA updates. On first boot, values are provisioned from a build-time `.env` embed; thereafter they are read from flash.

```rust
// src/config.rs
use esp_idf_svc::nvs::{EspDefaultNvsPartition, EspNvs, NvsDefault};
use heapless::String;

pub struct Config {
    pub wifi_ssid:       String<64>,
    pub wifi_pass:       String<64>,
    pub telegram_token:  String<128>,
    pub deepseek_key:    String<128>,
    pub allowed_user_id: i64,
}

impl Config {
    pub fn load(nvs_part: &EspDefaultNvsPartition) -> anyhow::Result<Self> {
        let nvs = EspNvs::new(nvs_part.clone(), "agent_cfg", true)?;
        Ok(Self {
            wifi_ssid:       nvs_str(&nvs, "wifi_ssid")?,
            wifi_pass:       nvs_str(&nvs, "wifi_pass")?,
            telegram_token:  nvs_str(&nvs, "tg_token")?,
            deepseek_key:    nvs_str(&nvs, "ds_key")?,
            allowed_user_id: nvs_i64(&nvs, "tg_uid")?,
        })
    }
}

fn nvs_str<const N: usize>(
    nvs: &EspNvs<NvsDefault>, key: &str
) -> anyhow::Result<String<N>> {
    let mut buf = [0u8; N];
    let len = nvs.get_raw(key, &mut buf)?.unwrap_or(0);
    Ok(String::from(core::str::from_utf8(&buf[..len])?))
}
```

### 6.3 wifi.rs — Wi-Fi Station Driver

```rust
// src/wifi.rs
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    nvs::EspDefaultNvsPartition,
    wifi::{BlockingWifi, ClientConfiguration, Configuration, EspWifi},
};
use esp_idf_hal::modem::Modem;

pub fn connect(
    cfg:     &crate::config::Config,
    sysloop: &EspSystemEventLoop,
    nvs:     &EspDefaultNvsPartition,
    modem:   Modem,
) -> anyhow::Result<BlockingWifi<EspWifi<'static>>> {
    let wifi_driver = EspWifi::new(modem, sysloop.clone(), Some(nvs.clone()))?;
    let mut wifi    = BlockingWifi::wrap(wifi_driver, sysloop.clone())?;

    wifi.set_configuration(&Configuration::Client(ClientConfiguration {
        ssid:     cfg.wifi_ssid.as_str().try_into().unwrap(),
        password: cfg.wifi_pass.as_str().try_into().unwrap(),
        ..Default::default()
    }))?;

    wifi.start()?;
    wifi.connect()?;
    wifi.wait_netif_up()?;
    log::info!("Wi-Fi connected — IP: {:?}",
        wifi.wifi().sta_netif().get_ip_info()?);
    Ok(wifi)
}
```

### 6.4 telegram/api.rs — Telegram Bot Polling

The Telegram module uses long-polling against the Bot API. It is intentionally synchronous (no async runtime) to minimise stack overhead on FreeRTOS threads.

```rust
// src/telegram/api.rs
use esp_idf_svc::http::client::{Configuration as HttpConfig, EspHttpConnection};
use embedded_svc::http::client::Client;
use heapless::String;

const BASE: &str = "https://api.telegram.org/bot";

pub struct TelegramApi<'a> {
    token:  &'a str,
    offset: i64,
}

impl<'a> TelegramApi<'a> {
    pub fn new(token: &'a str) -> Self {
        Self { token, offset: 0 }
    }

    /// Blocking long-poll (timeout = 30 s). Returns vec of (chat_id, text).
    pub fn get_updates(&mut self) -> heapless::Vec<(i64, String<256>), 8> {
        let url = format!(
            "{}{}/getUpdates?timeout=30&offset={}",
            BASE, self.token, self.offset
        );
        let mut client = Client::wrap(
            EspHttpConnection::new(&HttpConfig {
                use_global_ca_store: true,
                crt_bundle_attach: Some(esp_idf_sys::esp_crt_bundle_attach),
                ..Default::default()
            }).unwrap(),
        );
        let _resp = client.get(&url).unwrap().submit().unwrap();
        // Parse JSON, extract message.text and chat.id
        // Update self.offset = last_update_id + 1
        todo!("parse response")
    }

    pub fn send_message(&self, chat_id: i64, text: &str) {
        // POST /sendMessage with JSON body
        todo!("send response")
    }
}
```

### 6.5 llm/deepseek.rs — LLM API Client

All LLM requests use the DeepSeek Chat Completions endpoint. The prompt is constructed from a system message that describes the ESP32 pin table, plus the user's instruction. The model responds with a structured JSON plan.

```rust
// src/llm/deepseek.rs
use serde_json::json;

pub struct DeepSeekClient<'a> {
    api_key: &'a str,
}

const SYSTEM_PROMPT: &str = r#"
You are an ESP32 GPIO controller. Given a natural-language task,
respond ONLY with a valid JSON object matching this schema:
{
  "actions": [
    {
      "pin": <GPIO number>,
      "mode": "output" | "input" | "pwm",
      "value": <0 or 1 for digital, 0-255 for pwm>,
      "duration_ms": <optional milliseconds>
    }
  ],
  "description": "<human-readable summary>"
}
Available GPIO: 2,4,5,12,13,14,15,16,17,18,19,21,22,23,25,26,27,32,33,34,35,36,39.
PWM-capable:    2,4,5,12,13,14,15,16,17,18,19,21,22,23,25,26,27,32,33.
Input-only (no output): 34,35,36,39.
Do not use strapping pins (0,2,15) for persistent output tasks.
"#;

impl<'a> DeepSeekClient<'a> {
    pub fn new(api_key: &'a str) -> Self {
        Self { api_key }
    }

    pub fn complete(
        &self,
        user_instruction: &str,
    ) -> anyhow::Result<crate::llm::plan::PinPlan> {
        let body = json!({
            "model": "deepseek-chat",
            "messages": [
                { "role": "system", "content": SYSTEM_PROMPT },
                { "role": "user",   "content": user_instruction }
            ],
            "response_format": { "type": "json_object" },
            "temperature": 0.1,
            "max_tokens": 512
        });
        // POST to https://api.deepseek.com/chat/completions
        // Parse response → choices[0].message.content
        // Deserialise → serde_json::from_str::<PinPlan>
        todo!("HTTP POST + parse")
    }
}
```

### 6.6 llm/plan.rs — Structured Plan Types

```rust
// src/llm/plan.rs
use serde::Deserialize;
use heapless::Vec;

#[derive(Debug, Deserialize)]
pub struct PinAction {
    pub pin:         u8,
    pub mode:        PinMode,
    pub value:       u8,
    pub duration_ms: Option<u32>,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PinMode {
    Output,
    Input,
    Pwm,
}

#[derive(Debug, Deserialize)]
pub struct PinPlan {
    pub actions:     Vec<PinAction, 16>,    // max 16 simultaneous actions
    pub description: heapless::String<256>,
}
```

### 6.7 gpio/registry.rs — Pin Capability Table

A compile-time static array encodes every ESP32 GPIO's capabilities. The executor validates LLM plans against this table before touching hardware — preventing, for instance, an LLM from trying to drive an input-only pin as output.

```rust
// src/gpio/registry.rs

#[derive(Copy, Clone)]
pub struct PinSpec {
    pub number:     u8,
    pub can_output: bool,
    pub can_pwm:    bool,
    pub can_input:  bool,
    pub strapping:  bool,   // avoid on boot
}

pub static PIN_TABLE: &[PinSpec] = &[
    PinSpec { number:  2, can_output: true,  can_pwm: true,  can_input: true,  strapping: true  },
    PinSpec { number:  4, can_output: true,  can_pwm: true,  can_input: true,  strapping: false },
    PinSpec { number:  5, can_output: true,  can_pwm: true,  can_input: true,  strapping: true  },
    PinSpec { number: 12, can_output: true,  can_pwm: true,  can_input: true,  strapping: true  },
    PinSpec { number: 13, can_output: true,  can_pwm: true,  can_input: true,  strapping: false },
    PinSpec { number: 14, can_output: true,  can_pwm: true,  can_input: true,  strapping: false },
    PinSpec { number: 15, can_output: true,  can_pwm: true,  can_input: true,  strapping: true  },
    PinSpec { number: 16, can_output: true,  can_pwm: true,  can_input: true,  strapping: false },
    PinSpec { number: 17, can_output: true,  can_pwm: true,  can_input: true,  strapping: false },
    PinSpec { number: 18, can_output: true,  can_pwm: true,  can_input: true,  strapping: false },
    PinSpec { number: 19, can_output: true,  can_pwm: true,  can_input: true,  strapping: false },
    PinSpec { number: 21, can_output: true,  can_pwm: true,  can_input: true,  strapping: false },
    PinSpec { number: 22, can_output: true,  can_pwm: true,  can_input: true,  strapping: false },
    PinSpec { number: 23, can_output: true,  can_pwm: true,  can_input: true,  strapping: false },
    PinSpec { number: 25, can_output: true,  can_pwm: true,  can_input: true,  strapping: false },
    PinSpec { number: 26, can_output: true,  can_pwm: true,  can_input: true,  strapping: false },
    PinSpec { number: 27, can_output: true,  can_pwm: true,  can_input: true,  strapping: false },
    PinSpec { number: 32, can_output: true,  can_pwm: true,  can_input: true,  strapping: false },
    PinSpec { number: 33, can_output: true,  can_pwm: true,  can_input: true,  strapping: false },
    PinSpec { number: 34, can_output: false, can_pwm: false, can_input: true,  strapping: false },
    PinSpec { number: 35, can_output: false, can_pwm: false, can_input: true,  strapping: false },
    PinSpec { number: 36, can_output: false, can_pwm: false, can_input: true,  strapping: false },
    PinSpec { number: 39, can_output: false, can_pwm: false, can_input: true,  strapping: false },
];

pub fn lookup(pin: u8) -> Option<PinSpec> {
    PIN_TABLE.iter().copied().find(|p| p.number == pin)
}
```

### 6.8 gpio/executor.rs — Plan Execution Engine

The executor receives a `PinPlan`, validates each action against the pin registry, then applies the state change using `esp-idf-hal` pin handles. Timed actions (`duration_ms`) schedule a reset via FreeRTOS timer.

```rust
// src/gpio/executor.rs
use esp_idf_hal::{gpio::*, prelude::*};
use crate::llm::plan::{PinMode, PinPlan};
use crate::gpio::registry;

pub fn apply(plan: &PinPlan) -> Result<(), &'static str> {
    log::info!("Executing plan: {}", plan.description);

    for action in &plan.actions {
        // 1. Validate against capability table
        let spec = registry::lookup(action.pin)
            .ok_or("unknown pin")?;

        match action.mode {
            PinMode::Output if !spec.can_output => return Err("pin cannot output"),
            PinMode::Pwm    if !spec.can_pwm    => return Err("pin not PWM capable"),
            PinMode::Input  if !spec.can_input  => return Err("pin cannot input"),
            _ => {}
        }

        // Warn if strapping pin used as persistent output
        if spec.strapping && action.mode == PinMode::Output {
            log::warn!("GPIO{} is a strapping pin — use with caution", action.pin);
        }

        // 2. Apply hardware state
        //    (pins are managed via GpioRegistry to avoid double-borrow)
        match action.mode {
            PinMode::Output => {
                log::info!("GPIO{} -> {}", action.pin, action.value);
                // set digital HIGH/LOW via PinDriver
            }
            PinMode::Pwm => {
                log::info!("GPIO{} PWM duty={}/255", action.pin, action.value);
                // configure LEDC channel, set duty cycle
            }
            PinMode::Input => {
                log::info!("GPIO{} reading...", action.pin);
                // read level and log/report back
            }
        }

        // 3. Schedule auto-reset if duration specified
        if let Some(ms) = action.duration_ms {
            log::info!("  auto-reset in {}ms", ms);
            // FreeRTOS one-shot timer → reset pin after ms
        }
    }
    Ok(())
}
```

---

## 7. End-to-End Message Flow

The following sequence illustrates the full lifecycle of a user command:

```
User (Telegram)
    │
    │  "Blink GPIO 2 at 1 Hz for 10 seconds"
    ▼
telegram_task  ──── long-poll getUpdates ────► Telegram API
    │
    │  validate user_id whitelist
    ▼
deepseek.rs  ──── POST /chat/completions ────► DeepSeek API
    │                                              │
    │  ◄──── JSON plan response ──────────────────┘
    ▼
plan.rs  ──── serde_json::from_str::<PinPlan>
    │
    │  validate against PIN_TABLE
    ▼
executor.rs  ──── apply GPIO state ────► ESP32 hardware
    │
    │  log to UART0 (Serial)
    ▼
telegram_task  ──── sendMessage ────────────► User (Telegram)
    │
    │  [after 10 s]
    ▼
FreeRTOS timer  ──── reset GPIO 2 to LOW
```

**Step-by-step:**

1. User sends: `"Blink GPIO 2 at 1 Hz for 10 seconds"` on Telegram
2. `telegram_task` receives the message via long-poll `getUpdates`
3. Message text is validated (user_id whitelist check)
4. `deepseek.rs` constructs the prompt (system + user instruction)
5. HTTP POST to `api.deepseek.com/chat/completions` with JSON mode
6. LLM responds with structured JSON plan `(pin=2, mode=pwm, value=128, duration_ms=10000)`
7. `plan.rs` deserialises JSON into `PinPlan` struct (zero heap allocation via heapless)
8. `PinPlan` is validated against `PIN_TABLE` (capability check)
9. `executor.rs` applies GPIO state; serial log printed to UART0
10. `telegram_task` sends confirmation: `"✅ Blinking GPIO 2 at 1 Hz for 10s"`
11. After 10 s, FreeRTOS timer fires → GPIO 2 reset to idle

> **📌 Note:** All plan details are printed to Serial before hardware changes, enabling safe review during development.

---

## 8. Serial Monitor Interface

The USB Serial interface mirrors the Telegram interface — every command sent via serial is processed through the same LLM pipeline. This enables local debugging without a live Telegram connection.

### 8.1 Serial Commands

| Command | Description |
|---------|-------------|
| `help` | Print available commands |
| `<natural language>` | Any free-text instruction — forwarded to LLM pipeline |
| `gpio list` | Print capability table for all GPIO |
| `gpio set <n> <0\|1>` | Directly override a digital output pin (bypasses LLM) |
| `gpio read <n>` | Read and print current level of a pin |
| `config set <k> <v>` | Update NVS config key (e.g. `config set wifi_ssid MyNet`) |
| `reset` | Software reset the ESP32 |
| `loglevel <level>` | Set log verbosity: `error` \| `warn` \| `info` \| `debug` |

### 8.2 Serial Monitoring

```bash
# Flash and open monitor in one step
cargo espflash flash --monitor

# Or monitor a running device (115200 baud default)
espflash monitor /dev/ttyUSB0

# Example serial session
ESP32-Agent ready. Type a task or 'help'.
> Turn on the red LED on pin 13
[LLM] Sending to DeepSeek...
[PLAN] {"actions":[{"pin":13,"mode":"output","value":1}],
        "description":"Set GPIO 13 HIGH to turn on LED"}
[GPIO] GPIO13 -> HIGH
Done.
```

---

## 9. Memory Management Strategy

### 9.1 Heap Usage Budget

ESP32 has ~320 KB of accessible DRAM (with BT disabled). The following allocation strategy targets under **200 KB** total heap usage:

| Component | Memory |
|-----------|--------|
| Wi-Fi + TCP/IP stack (LwIP) | ~80 KB (managed by ESP-IDF) |
| TLS session (HTTPS to Telegram/DeepSeek) | ~36 KB peak, freed after request |
| HTTP response buffer (LLM JSON response) | ~4 KB static |
| `heapless::Vec` / `String` allocations | ~2 KB (stack-allocated, zero heap) |
| FreeRTOS task stacks | ~20 KB (4 tasks × 5 KB) |
| **Remaining for user data** | **~178 KB free** |

### 9.2 Rust Ownership for Pin Safety

Rust's ownership system prevents double-use of GPIO handles at **compile time**. A `GpioRegistry` struct owns all pin handles; methods return exclusive mutable references, ensuring the same pin cannot be simultaneously driven as output and input — a class of bugs impossible to catch at runtime in C.

```rust
// Zero-cost abstraction — ownership enforced at compile time
pub struct GpioRegistry {
    // Only one owner of each pin handle at a time
    pin2:  Option<PinDriver<'static, Gpio2,  Output>>,
    pin4:  Option<PinDriver<'static, Gpio4,  Output>>,
    pin13: Option<PinDriver<'static, Gpio13, Output>>,
    // ... all GPIO as Options, initialised on first use
}

impl GpioRegistry {
    pub fn claim_output(
        &mut self,
        pin: u8,
    ) -> Option<&mut dyn embedded_hal::digital::OutputPin> {
        match pin {
            2  => self.pin2.as_mut().map(|p| p as &mut dyn _),
            4  => self.pin4.as_mut().map(|p| p as &mut dyn _),
            13 => self.pin13.as_mut().map(|p| p as &mut dyn _),
            _  => None,
        }
    }
}
```

### 9.3 Stack-Only Strings with `heapless`

All string buffers (API responses, log messages, command parsing) use `heapless::String<N>` — fixed-capacity strings allocated on the stack. This eliminates heap fragmentation for the most frequent operations in the main loop.

```rust
use heapless::String;

// No heap allocation — lives on the FreeRTOS task stack
let mut cmd: String<256> = String::new();
cmd.push_str("Turn on LED at GPIO 13").unwrap();
```

### 9.4 Static Dispatch over Dynamic Dispatch

Where possible, the codebase uses generics and monomorphisation rather than `dyn Trait` to avoid vtable overhead and keep code in flash (IRAM) minimal.

```rust
// Preferred: zero-cost static dispatch
fn send_plan<C: LlmClient>(client: &C, instruction: &str) -> PinPlan {
    client.complete(instruction).unwrap()
}

// Avoid unless runtime polymorphism is required
fn send_plan_dyn(client: &dyn LlmClient, instruction: &str) -> PinPlan { ... }
```

---

## 10. Security Considerations

### 10.1 Telegram User Whitelist

Only Telegram user IDs explicitly stored in NVS (`allowed_user_id`) can trigger GPIO changes. All other messages are silently discarded. The whitelist can be extended to a `heapless::Vec<i64, 8>` for multi-user scenarios.

```rust
if update.message.from.id != cfg.allowed_user_id {
    log::warn!("Rejected message from user {}", update.message.from.id);
    return;   // silently discard
}
```

### 10.2 TLS Certificate Validation

All HTTPS requests (Telegram API, DeepSeek API) use ESP-IDF's bundled CA certificate chain (`esp_crt_bundle_attach`). Self-signed or invalid certificates cause the request to fail with a TLS handshake error — preventing MITM attacks on the LLM pipeline.

```rust
EspHttpConnection::new(&HttpConfig {
    use_global_ca_store: true,
    crt_bundle_attach: Some(esp_idf_sys::esp_crt_bundle_attach),
    ..Default::default()
})
```

### 10.3 LLM Plan Sandboxing

The executor enforces three layers of safety before any hardware action:

1. **Schema validation** — `PinPlan` must deserialise correctly or the plan is rejected
2. **Capability check** — each action validated against the static `PIN_TABLE`
3. **Strapping pin guard** — GPIO 0, 2, 15 log a warning when set as persistent output

> **📌 Note:** The LLM plan is always printed to Serial before execution. This allows a developer watching the monitor to catch unexpected pin assignments during testing.

### 10.4 API Key Storage

The DeepSeek API key is stored in ESP32 NVS flash, which is not accessible via JTAG debug port by default (eFuse read-protection can be enabled for production). The key is **never** logged or included in Telegram replies.

### 10.5 Rate Limiting

To prevent runaway API costs from bot spam, a simple token bucket rate limiter is applied in `telegram/mod.rs`:

```rust
const MAX_REQUESTS_PER_MINUTE: u8 = 10;

// If bucket exhausted, reply with rate-limit message without calling LLM
if !rate_limiter.try_consume() {
    api.send_message(chat_id, "⚠️ Rate limit reached. Please wait.");
    return;
}
```

---

## 11. Usage Examples

### 11.1 Telegram Command Examples

| Instruction | Resulting Action |
|-------------|-----------------|
| `Turn on GPIO 13` | Sets GPIO 13 HIGH (LED on) |
| `Blink GPIO 2 at 2 Hz` | PWM square wave at 50% duty, 2 Hz on GPIO 2 |
| `Read temperature on GPIO 34` | Configure GPIO 34 as input; read raw ADC |
| `Pulse GPIO 5 for 500ms` | GPIO 5 HIGH for 500 ms, then auto-reset LOW |
| `Set GPIO 18 PWM to 75%` | LEDC duty cycle = 192/255 on GPIO 18 |
| `Turn off all outputs` | Reset all output pins to LOW |
| `What pins are available?` | LLM replies with capability summary |
| `Servo sweep on GPIO 21` | LLM generates PWM ramp 0→180° on GPIO 21 |
| `Flash SOS on GPIO 2` | LLM encodes morse SOS blink sequence |

### 11.2 Example LLM JSON Plan

```json
// User: "Blink the blue LED on GPIO 2 three times, 200ms each"
// LLM response (DeepSeek JSON mode):
{
  "actions": [
    { "pin": 2, "mode": "output", "value": 1, "duration_ms": 200 },
    { "pin": 2, "mode": "output", "value": 0, "duration_ms": 200 },
    { "pin": 2, "mode": "output", "value": 1, "duration_ms": 200 },
    { "pin": 2, "mode": "output", "value": 0, "duration_ms": 200 },
    { "pin": 2, "mode": "output", "value": 1, "duration_ms": 200 },
    { "pin": 2, "mode": "output", "value": 0 }
  ],
  "description": "Blink GPIO 2 three times with 200 ms on/off cycles"
}
```

### 11.3 Multi-Pin Example

```json
// User: "Turn on the fan (GPIO 26) and set the indicator LED (GPIO 13) to 50% brightness"
{
  "actions": [
    { "pin": 26, "mode": "output", "value": 1 },
    { "pin": 13, "mode": "pwm",    "value": 128 }
  ],
  "description": "Activate fan relay on GPIO 26 and dim indicator LED on GPIO 13 to 50%"
}
```

---

## 12. Build, Flash & Monitor

### 12.1 First-Time Credential Provisioning

```bash
# Provision via Serial CLI after first flash
> config set wifi_ssid    MyNetwork
> config set wifi_pass    MyPassword
> config set tg_token     123456:ABCdef...
> config set ds_key       sk-deepseek-...
> config set tg_uid       987654321
> reset
```

### 12.2 Build & Flash

```bash
# Debug build (faster compile, larger binary)
cargo espflash flash --port /dev/ttyUSB0

# Release build (optimised for size — use for deployment)
cargo espflash flash --release --port /dev/ttyUSB0

# Flash and immediately open serial monitor
cargo espflash flash --release --monitor --port /dev/ttyUSB0

# Check binary size breakdown
cargo size --release -- -A

# Typical release binary size: ~800 KB (within 4 MB flash)
```

### 12.3 Partition Table

```csv
# partitions.csv
# Name,    Type, SubType, Offset,   Size,    Flags
nvs,       data, nvs,     0x9000,   0x6000,
otadata,   data, ota,     0xf000,   0x2000,
phy_init,  data, phy,     0x11000,  0x1000,
factory,   app,  factory, 0x10000,  1M,
ota_0,     app,  ota_0,   ,         1M,
ota_1,     app,  ota_1,   ,         1M,
```

### 12.4 OTA Updates (Optional)

With a dual OTA partition layout, the bot can accept a `/ota <url>` command to download and apply firmware over-the-air — enabling field updates without physical access.

```rust
// Handle /ota command in telegram/mod.rs
if text.starts_with("/ota ") {
    let url = &text[5..];
    ota::update_from_url(url)?;
}
```

---

## 13. Extending ESP32-Agent

### 13.1 Adding a New LLM Backend

The `LlmClient` trait abstracts the LLM backend. Implement it for any provider (OpenAI, Anthropic, Ollama, Gemini, etc.):

```rust
// src/llm/mod.rs
pub trait LlmClient {
    fn complete(&self, instruction: &str) -> anyhow::Result<crate::llm::plan::PinPlan>;
}

// Swap the backend in main.rs without touching the rest of the codebase:
// let llm = llm::openai::OpenAiClient::new(&cfg.openai_key);
// let llm = llm::deepseek::DeepSeekClient::new(&cfg.deepseek_key);
// let llm = llm::ollama::OllamaClient::new("http://192.168.1.100:11434");
```

### 13.2 Adding MQTT Transport

Replace or supplement the Telegram polling loop with an MQTT subscriber using `esp-mqtt`. Subscribe to a topic (`esp32/agent/command`), process messages through the same LLM pipeline, and publish results to `esp32/agent/response`.

```rust
// Pseudo-code for MQTT integration
let mut mqtt = EspMqttClient::new(broker_url, &mqtt_cfg, move |event| {
    if let MqttEvent::Received(msg) = event {
        let plan = llm_client.complete(msg.payload_str());
        executor::apply(&plan);
        mqtt.publish("esp32/agent/response", &plan.description);
    }
})?;
mqtt.subscribe("esp32/agent/command", QoS::AtLeastOnce)?;
```

### 13.3 Adding I2C / SPI Peripherals

Extend the system prompt with a `PeripheralMap` that describes attached sensors. The LLM can then route instructions like `"read the humidity sensor"` to the correct I2C address and GPIO bus.

```rust
// Append to SYSTEM_PROMPT dynamically at runtime
let peripheral_ctx = format!(
    "Attached peripherals: SHT31 humidity sensor on I2C (SDA=GPIO21, SCL=GPIO22, addr=0x44). \
     DS18B20 temperature on GPIO4 (1-Wire)."
);
```

### 13.4 Adding Voice Commands

Route Telegram voice messages through the Whisper API before sending to the LLM pipeline:

```
Voice note (OGG) → Whisper transcription → text → DeepSeek → PinPlan → GPIO
```

---

## 14. Troubleshooting

| Symptom | Likely Cause | Fix |
|---------|-------------|-----|
| `Guru Meditation: LoadProhibited` | Stack overflow in task | Increase `stack_size` in `Builder::new()` |
| TLS handshake failed | System time not set (cert expiry check fails) | Enable SNTP: `CONFIG_LWIP_SNTP=y` in sdkconfig |
| LLM returns invalid JSON | Model confused by complex instruction | Simplify instruction; lower temperature to `0.05` |
| `gpio plan rejected: 'pin cannot output'` | LLM chose input-only pin (34/35/36/39) | Rephrase or explicitly name a valid output pin |
| Wi-Fi disconnects frequently | RSSI too low or DHCP lease expiry | Add reconnect loop with exponential backoff |
| OOM panic during TLS | Insufficient DRAM for TLS session | Enable PSRAM in sdkconfig or disable Bluetooth |
| Telegram bot not responding | Long-poll timeout on slow networks | Reduce timeout param to 15 s; add retry logic |
| `cargo build` fails: xtensa target not found | `espup` not sourced | Run: `. ~/export-esp.sh` |
| Binary too large for flash | Debug build with all features | Use `--release` and check `cargo size` |
| NVS read returns empty | First boot, NVS partition not initialised | Provision via Serial CLI then reset |

---

## 15. Roadmap

- **v1.1** — OTA firmware update via Telegram `/ota` command
- **v1.1** — MQTT transport layer (AWS IoT / Mosquitto)
- **v1.2** — Peripheral map: I2C sensor auto-discovery in LLM context
- **v1.2** — Voice-to-text via Whisper API (audio message → Telegram)
- **v1.3** — Multi-ESP32 mesh: one bot controlling a fleet of nodes
- **v1.3** — Web dashboard (ESP32 HTTP server) for plan history and pin state
- **v2.0** — ESP32-S3 with USB-OTG support + native USB Serial
- **v2.0** — On-device inference: small LLM running locally via llama.cpp Xtensa port

---

## 16. Glossary

| Term | Definition |
|------|-----------|
| GPIO | General Purpose Input/Output — digital/analog pin |
| NVS | Non-Volatile Storage — ESP32 key-value flash store |
| LEDC | LED Control peripheral — ESP32 PWM engine (up to 16 channels) |
| FreeRTOS | Real-time OS running under ESP-IDF; manages tasks and timers |
| LLM | Large Language Model — AI model that processes natural language |
| `heapless` | Rust crate providing fixed-capacity, stack-allocated collections |
| `esp-idf-hal` | Hardware Abstraction Layer crate for ESP-IDF peripherals in Rust |
| `PinPlan` | Structured JSON + Rust struct describing GPIO actions from the LLM |
| Strapping pin | GPIO sampled at boot to configure chip mode; avoid for general use |
| PSRAM | Pseudo-Static RAM — external SPI RAM chip on some ESP32 modules |
| IRAM | Instruction RAM — fast on-chip memory for time-critical ISR code |
| OTA | Over-The-Air — firmware update delivered via Wi-Fi without USB |

---

## 17. References & Further Reading

- **esp-idf-hal crate** — https://github.com/esp-rs/esp-idf-hal
- **esp-idf-svc crate** — https://github.com/esp-rs/esp-idf-svc
- **The Rust on ESP Book** — https://esp-rs.github.io/book/
- **DeepSeek API Documentation** — https://platform.deepseek.com/api-docs
- **Telegram Bot API** — https://core.telegram.org/bots/api
- **heapless crate** — https://docs.rs/heapless
- **OpenClaw project (inspiration)** — https://github.com/openagent/openclaw
- **ESP32 Technical Reference Manual** — https://www.espressif.com/en/support/documents/technical-documents
- **cargo-espflash** — https://github.com/esp-rs/espflash
- **espup (toolchain installer)** — https://github.com/esp-rs/espup

---

*ESP32-Agent · hexcodeplus· 2026 · MIT License*