# ESP32-CLAW Implementation Plan — RTOS-Centric
## Derived from document.md v1.0 · FreeRTOS-First Architecture

---

## Design Philosophy

Every component is modeled as a **FreeRTOS task** or **ISR**. There is no main-loop polling. Communication between layers happens exclusively via **FreeRTOS Queues**, **Event Groups**, and **Timers**. Shared mutable state is protected by **Mutexes** or owned by a single task with queue-based access. This eliminates data races, makes timing deterministic, and mirrors the architecture described in §6.1.

---

## Phase 1: Toolchain & RTOS-Aware Project Bootstrap
**Goal:** Buildable Rust-on-ESP32 project with FreeRTOS static allocation enabled.

| # | Task | Notes |
|---|------|-------|
| 1.1 | Install Espressif Rust toolchain (`espup`) and source environment | Requires `export-esp.sh` in shell rc |
| 1.2 | Install ESP-IDF v5.1.3 and run `./install.sh esp32` | Set `IDF_PATH` |
| 1.3 | Install `cargo-espflash` v3.x | For flashing & monitoring |
| 1.4 | Create project root with `Cargo.toml`, `.cargo/config.toml`, `sdkconfig.defaults` | Target: `xtensa-esp32-espidf` |
| 1.5 | Add dependencies: `esp-idf-sys`, `esp-idf-hal`, `esp-idf-svc`, `embedded-hal`, `embedded-svc`, `serde` (derive, alloc), `serde_json` (alloc), `heapless`, `log`, `anyhow` | See doc §5.1 |
| 1.6 | **Configure `sdkconfig.defaults` for FreeRTOS static allocation & minimal heap** | `CONFIG_FREERTOS_SUPPORT_STATIC_ALLOCATION=y`, `CONFIG_FREERTOS_UNICORE=n` (use both cores) |
| 1.7 | Configure release profile: `opt-level="z"`, `lto=true`, `codegen-units=1` | Minimise flash usage |
| 1.8 | Write `build.rs` for `esp-idf-sys` binding generation | Standard esp-rs pattern |
| 1.9 | Verify with `cargo build` (expect linker errors until main.rs exists) | Validate toolchain |

**RTOS Decisions:**
- Static task creation preferred; dynamic only for TLS scratch buffers.
- Dual-core: Wi-Fi/Network on PRO (CPU 0), Application tasks on APP (CPU 1) where possible.

---

## Phase 2: RTOS Task Foundation — Config, Wi-Fi, Serial
**Goal:** NVS config, Wi-Fi state machine via Event Group, Serial task, and basic task architecture.

| # | Task | Notes |
|---|------|-------|
| 2.1 | Implement `src/config.rs` — NVS-backed `Config` struct | Keys: `wifi_ssid`, `wifi_pass`, `tg_token`, `ds_key`, `tg_uid`. Use `heapless::String<N>`. First boot fallback from `.env` embed |
| 2.2 | Define **FreeRTOS Event Group** bits in `src/rtos.rs` | `BIT_WIFI_CONNECTED`, `BIT_WIFI_FAIL`, `BIT_IP_OBTAINED`. Central state signaling |
| 2.3 | Implement `src/wifi.rs` — Wi-Fi station task + event handler | Uses `esp_idf_svc::wifi::EspWifi` with `sysloop` subscribe. On `STA_CONNECTED` → set `BIT_WIFI_CONNECTED`. On `GOT_IP` → set `BIT_IP_OBTAINED`. Blocking wait with `xEventGroupWaitBits` |
| 2.4 | Implement `src/serial.rs` as a **dedicated FreeRTOS task** (`serial_task`) | Reads UART0 via `uart::UartDriver`. Parses commands. Routes natural-language text to a **shared command queue** |
| 2.5 | Implement `src/main.rs` entry point as the **supervisor / root task** | Initialise logger, take peripherals/NVS/sysloop, load config, create Event Group, spawn `wifi_task`, spawn `serial_task` |
| 2.6 | Add `partitions.csv` with NVS + factory + dual OTA slots | Factory 1M, OTA0/OTA1 1M each |
| 2.7 | Test: Flash, open monitor, run `config set` and `reset`, verify NVS persistence | Confirm Wi-Fi connects if credentials valid. Verify Event Group bits toggle |

**RTOS Decisions:**
- `wifi_task` priority = 5 (network is critical).
- `serial_task` priority = 3 (local debug, can preempt app logic).
- Root task (`app_main`) becomes an **init task** that deletes itself after spawning children.

---

## Phase 3: GPIO Subsystem — Queue-Driven Executor
**Goal:** Safe, validated GPIO control with single-owner executor task.

| # | Task | Notes |
|---|------|-------|
| 3.1 | Implement `src/gpio/registry.rs` — static `PIN_TABLE` array | 23 pins (2,4,5,12-19,21-23,25-27,32-36,39). Flags: `can_output`, `can_pwm`, `can_input`, `strapping`. `lookup(pin)` helper |
| 3.2 | Implement `src/gpio/mod.rs` — `GpioRegistry` owning all pin handles | `Option<PinDriver<'static, GpioX, Output>>` per pin. Wrapped in a **FreeRTOS Mutex** if any other task needs inspection (otherwise owned solely by executor) |
| 3.3 | Define **FreeRTOS Queue** `xGpioQueue` in `src/rtos.rs` | Capacity 4, item size = `PinPlan` (stack-allocated). Producer: Telegram/Serial tasks. Consumer: `gpio_executor_task` |
| 3.4 | Implement `src/gpio/executor.rs` — `gpio_executor_task` | Loop forever on `xQueueReceive(xGpioQueue, ...)`. Validate each action against `PIN_TABLE`, warn on strapping pins, dispatch Output/PWM/Input via `GpioRegistry`. Log every action |
| 3.5 | Implement **FreeRTOS Software Timers** for `duration_ms` auto-reset | `xTimerCreate` one-shot per timed action. Callback resets pin to idle state. Use static timer buffer |
| 3.6 | Wire serial `gpio` commands directly to `GpioRegistry` (bypass LLM) | For local debugging — sends directly to `xGpioQueue` or calls registry under mutex |
| 3.7 | Test: `gpio list`, `gpio set 13 1`, `gpio read 34`, verify hardware | Measure with multimeter/LED. Verify timer resets after `duration_ms` |

**RTOS Decisions:**
- `gpio_executor_task` priority = 4 (time-sensitive hardware).
- Only the executor task ever mutates `GpioRegistry`. No mutex needed on mutations; queue provides serialization.
- Timer service task priority = 6 (higher than executor to guarantee reset deadlines).

---

## Phase 4: LLM Integration — Synchronous HTTP in Dedicated Task
**Goal:** DeepSeek client, plan types, and JSON parsing — isolated to one task to manage TLS heap spikes.

| # | Task | Notes |
|---|------|-------|
| 4.1 | Implement `src/llm/plan.rs` — `PinAction`, `PinMode`, `PinPlan` structs | `heapless::Vec<PinAction, 16>` and `heapless::String<256>` for description |
| 4.2 | Implement `src/llm/deepseek.rs` — `DeepSeekClient` | `complete(instruction) -> anyhow::Result<PinPlan>`. HTTP POST to `api.deepseek.com/chat/completions`. System prompt includes pin table |
| 4.3 | Implement `src/llm/mod.rs` — `LlmClient` trait | `fn complete(&self, instruction: &str) -> anyhow::Result<PinPlan>` |
| 4.4 | Add TLS CA bundle config to HTTP clients | `use_global_ca_store: true`, `crt_bundle_attach: Some(esp_crt_bundle_attach)` |
| 4.5 | Add `SYSTEM_PROMPT` constant with full GPIO capability description | See doc §6.5 |
| 4.6 | **LLM calls run synchronously inside `telegram_task` / `serial_task`** (no separate LLM task) | TLS + HTTPS heap spike (~36 KB) is temporary and freed after `complete()` returns. Avoids another large stack task |
| 4.7 | Test: Send a hardcoded instruction from serial, print raw JSON and parsed `PinPlan` | Validate serde + heapless deserialisation |

**RTOS Decisions:**
- No persistent LLM task. HTTPS client is stack-local inside the caller task; heap spike is bounded and released on function return.
- If OOM observed, spawn a dedicated `llm_worker_task` with an 8192-byte stack and its own queue.

---

## Phase 5: Telegram Bot Transport — Polling Task + Rate Limiter
**Goal:** Long-polling bot with user whitelist and rate limiting, fully async relative to other tasks via queues.

| # | Task | Notes |
|---|------|-------|
| 5.1 | Implement `src/telegram/types.rs` — minimal Telegram structs | `Update`, `Message`, `Chat`, `User` with needed fields |
| 5.2 | Implement `src/telegram/api.rs` — `TelegramApi` | `get_updates()` long-poll (30s timeout), `send_message()`. Returns `heapless::Vec<(i64, String<256>), 8>` |
| 5.3 | Implement `src/telegram/mod.rs` — **`telegram_task`** (FreeRTOS task) | Loop: `get_updates` → whitelist check (`allowed_user_id`) → rate limiter → call `llm.complete()` → send `PinPlan` to `xGpioQueue` → `send_message` confirmation |
| 5.4 | Implement **rate limiter** as a token bucket in `telegram/mod.rs` | `MAX_REQUESTS_PER_MINUTE = 10`. Refill via `xTaskGetTickCount()` interval. No separate task needed |
| 5.5 | Integrate `telegram_task` in `main.rs` | Shares `xGpioQueue` producer with serial. Waits on `BIT_WIFI_CONNECTED` + `BIT_IP_OBTAINED` before first poll |
| 5.6 | Test: Send Telegram message, observe DeepSeek call, observe GPIO action, receive confirmation reply | End-to-end smoke test |

**RTOS Decisions:**
- `telegram_task` priority = 3 (network I/O bound, not latency-critical).
- `telegram_task` stack size = 8192 (HTTPS + JSON parsing + Telegram API buffers).
- Block on `xEventGroupWaitBits(WIFI_CONNECTED_BIT | IP_OBTAINED_BIT, pdTRUE, pdTRUE, portMAX_DELAY)` before entering poll loop.

---

## Phase 6: RTOS Synchronization, Safety & Hardening
**Goal:** Production-ready reliability with explicit RTOS primitives.

| # | Task | Notes |
|---|------|-------|
| 6.1 | Add strapping-pin warnings in executor logs | GPIO 0, 2, 15 caution |
| 6.2 | Add schema validation layer before executor | Reject malformed `PinPlan` at deserialisation |
| 6.3 | Implement **`duration_ms` support with FreeRTOS Software Timers** | `xTimerCreateStatic` one-shot per timed action. Callback posts a "reset" message to `xGpioQueue` or directly calls registry (if ISR-safe) |
| 6.4 | Add `loglevel` serial command integration with `esp_idf_svc::log::EspLogger` | Runtime verbosity change |
| 6.5 | Add **Wi-Fi reconnect task** with exponential backoff | Monitor `BIT_WIFI_FAIL`. Backoff capped at 60s. Uses `vTaskDelay` |
| 6.6 | Add retry logic to Telegram `getUpdates` on network errors | Reduce timeout to 15s on slow networks; `vTaskDelay(5000)` between retries |
| 6.7 | Add `cargo size --release` check to CI/build script | Ensure binary stays under ~1 MB |
| 6.8 | Implement **Watchdog awareness** — `gpio_executor_task` and `telegram_task` feed `esp_task_wdt` | Call `esp_task_wdt_reset()` or ensure tasks block on queues/IO, not spin |
| 6.9 | Document first-time provisioning flow in README | Serial CLI `config set` commands |

**RTOS Decisions:**
- Timer service task stack in `sdkconfig.defaults`: `CONFIG_FREERTOS_TIMER_TASK_STACK_DEPTH=4096`.
- Use `heapless::spsc::Queue` for Plan queue (zero heap), but wrap producer side reference in a **Critical Section** or **Mutex** if both Telegram and Serial can enqueue simultaneously.
- Alternatively, use FreeRTOS `xQueue` directly (safer for multi-producer) and define a C-compatible `PinPlan` struct.

---

## Phase 7: Validation & Release
**Goal:** Confirm all documented features work on real hardware under RTOS scheduling.

| # | Task | Notes |
|---|------|-------|
| 7.1 | Run all Telegram examples from doc §11.1 | Turn on, blink, PWM, pulse, read, multi-pin |
| 7.2 | Run all Serial commands from doc §8.1 | Verify local debug parity |
| 7.3 | Monitor heap usage during TLS + LLM request | Confirm under 200 KB total. Use `esp_get_free_heap_size()` logged periodically |
| 7.4 | Verify rate limiter blocks >10 req/min | Protect API costs |
| 7.5 | Verify unknown Telegram user ID is silently rejected | Security check |
| 7.6 | Flash release build, confirm OTA partition layout valid | `cargo espflash flash --release` |
| 7.7 | Stress test: rapid-fire 20 Telegram commands | Verify queue doesn't overflow (capacity 4). Confirm backpressure/rejection |
| 7.8 | Verify timer accuracy: `duration_ms=500` five times | Oscilloscope or logic analyzer on GPIO |
| 7.9 | Tag v1.0, write CHANGELOG | Release ready |

---

## RTOS Task Map (Reference)

| Task Name | Priority | Stack | Core | Responsibility |
|-----------|----------|-------|------|----------------|
| `app_main` (init) | 1 | 4096 | Any | Boot, spawn tasks, self-delete |
| `wifi_task` | 5 | 4096 | CPU 0 | Connect, maintain, signal Event Group |
| `telegram_task` | 3 | 8192 | CPU 1 | Poll Telegram, call LLM, enqueue plans |
| `serial_task` | 3 | 4096 | CPU 1 | UART CLI, enqueue plans, direct registry ops |
| `gpio_executor_task` | 4 | 4096 | CPU 1 | Dequeue plans, validate, apply hardware |
| Timer Service | 6 | 4096 | Any | FreeRTOS software timers (auto-reset) |
| IDLE (x2) | 0 | 1024 | Each | Heap cleanup, WDT pet |

---

## Inter-Task Communication Map

| Primitive | Name | Users | Purpose |
|-----------|------|-------|---------|
| Event Group | `xSystemEventGroup` | Wi-Fi → Telegram, Serial | Signal network readiness |
| Queue | `xGpioQueue` | Telegram, Serial → Executor | Deliver `PinPlan` structs |
| Mutex | `xGpioRegistryMutex` | Serial (optional) | Safe inspection of pin state |
| Timer (one-shot) | `xPinResetTimer[N]` | Executor → ISR/Callback | Auto-reset pin after `duration_ms` |

---

## Risk Mitigation

| Risk | Mitigation |
|------|------------|
| TLS handshake fails | Enable SNTP in `sdkconfig.defaults` |
| Stack overflow (Guru Meditation) | Static analysis of task stack sizes; start high, measure with `uxTaskGetStackHighWaterMark` |
| LLM returns invalid JSON | Lower temperature to 0.05; print raw response to serial |
| OOM during HTTPS | Disable Bluetooth in sdkconfig; enable PSRAM if available; isolate TLS to one task's lifetime |
| Binary too large | Always use `--release`; audit with `cargo size` |
| Queue overflow under load | Capacity 4 with backpressure: reply "Busy" if `xQueueSend` fails |
| Priority inversion on GPIO | Executor owns registry; no mutex on mutations. Queue serializes all access. |

---

## Post-v1.0 Roadmap (Backlog)

- OTA via `/ota <url>` command (dedicated `ota_task` with Event Group signal)
- MQTT transport layer (replace/supplement Telegram with `mqtt_task` subscribing to topics)
- I2C peripheral map in LLM context (`i2c_task` reading sensors on request)
- Voice-to-text via Whisper API (audio download → `whisper_task`)
- Multi-ESP32 mesh control (`mesh_task` using ESP-MESH)
- Web dashboard for pin state/history (`http_server_task`)
- ESP32-S3 USB-OTG + native serial
- On-device LLM inference (llama.cpp Xtensa — isolate to a low-priority `inference_task`)
