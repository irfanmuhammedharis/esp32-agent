# ESP32-CLAW

**AI-Driven GPIO Controller for ESP32 — Powered by Rust · Telegram · DeepSeek LLM**

> Send plain-English instructions to your ESP32. A DeepSeek LLM translates intent into hardware actions. No recompilation. No manual pin mapping. The AI has absolute control.

---

## What It Does

ESP32-CLAW is a self-programming firmware for the ESP32. You describe what you want in natural language — *"Blink the red LED on GPIO 13"* or *"Read soil moisture on GPIO 34 and turn on the pump if it's dry"* — and the onboard VM executes real hardware loops using:

- **Digital output** (`set_high` / `set_low`)
- **Digital input** (`read_digital`)
- **ADC reads** (`read_adc`) — 12-bit, GPIO 32-39
- **PWM output** (`set_pwm`) — LEDC hardware, 0-255 duty

All instructions flow through a DeepSeek LLM that generates a bytecode program (setup + loop + jumps + variables), which the on-device VM runs forever.

---

## Quick Start

### 1. Flash

```bash
cargo espflash flash --release --port /dev/ttyUSB0 --monitor
```

### 2. Provision (first boot only)

Over the serial monitor, set your credentials:

```
> config set wifi_ssid  MyHomeWiFi
> config set wifi_pass  MyPassword
> config set tg_token   123456789:ABCdef...
> config set ds_key     sk-deepseek-...
> config set tg_uid     123456789
> reset
```

| Key | How to get it |
|-----|--------------|
| `wifi_ssid` / `wifi_pass` | Your 2.4 GHz Wi-Fi |
| `tg_token` | Message [@BotFather](https://t.me/botfather) |
| `ds_key` | Register at [platform.deepseek.com](https://platform.deepseek.com) |
| `tg_uid` | Message [@userinfobot](https://t.me/userinfobot) |

### 3. Control

**Serial:** Type natural language at the `> ` prompt.  
**Telegram:** Message your bot. Only your `tg_uid` is whitelisted.

---

## Architecture

```
User (Telegram / Serial)
    │
    ▼
Telegram Task / Serial Task
    │
    ▼
DeepSeekClient::complete()  ──►  JSON Project
    │
    ▼
On-Device VM (program.rs)
    │
    ├── Setup phase  →  config pins
    └── Loop phase   →  read_adc, set_pwm, jump_if_lt, ...
    │
    ▼
GpioRegistry  ──►  Real ESP32 hardware
```

### VM Instruction Set

The LLM outputs a `Project` with:

| Instruction | Action |
|-------------|--------|
| `config_output` / `config_input` / `config_pwm` | Pin mode setup |
| `set_high` / `set_low` | Digital output |
| `set_pwm` | LEDC PWM duty (0-255) |
| `read_adc` | 12-bit ADC read → variable |
| `read_digital` | Digital input → variable |
| `set_var` | Store integer |
| `jump_if_lt` / `gt` / `eq` | Conditional branching |
| `delay` | Millisecond sleep |

---

## Hardware

| Pin | Output | PWM | Input | ADC | Notes |
|-----|--------|-----|-------|-----|-------|
| 2, 4, 5, 12-23, 25-27, 32-33 | ✅ | ✅ | ✅ | ✅* | Dual-mode I/O |
| 34, 35, 36, 39 | ❌ | ❌ | ✅ | ✅ | Input-only |

\* ADC on GPIO 32-39 (ADC1). ADC2 pins are avoided because they conflict with Wi-Fi.

---

## Tech Stack

- **Language:** Rust (Xtensa ESP32 toolchain)
- **HAL:** `esp-idf-hal` 0.45
- **RTOS:** FreeRTOS (via ESP-IDF v5.1)
- **Networking:** Wi-Fi station + TLS (mbedtls)
- **Storage:** ESP32 NVS flash
- **LLM:** DeepSeek Chat Completions (JSON mode)
- **Collections:** `heapless` (zero heap fragmentation)

---

## Repo Layout

```
├── src/
│   ├── main.rs           # Entry point, task spawning
│   ├── config.rs         # NVS-backed config store
│   ├── wifi.rs           # Wi-Fi station driver
│   ├── gpio/
│   │   ├── mod.rs        # GpioRegistry (real ADC + PWM + I/O)
│   │   └── registry.rs   # Pin capability table
│   ├── llm/
│   │   ├── deepseek.rs   # DeepSeek HTTP client
│   │   └── plan.rs       # Project schema & instructions
│   ├── telegram/         # Bot polling & API
│   ├── serial.rs         # USB-UART CLI
│   └── program.rs        # On-device VM interpreter
├── Cargo.toml
├── sdkconfig.defaults
├── partitions.csv
├── document.md           # Full architecture docs
└── USER_GUIDE.md         # End-user manual
```

---

## License

MIT — see [LICENSE](LICENSE). Free to use, modify, and distribute.
