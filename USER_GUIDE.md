# ESP32-Agent User Guide
## AI-Driven GPIO Controller for ESP32

---

## What is ESP32-Agent?

ESP32-Agent is a **self-programming GPIO controller**. You send it plain English instructions like *"Turn on the red LED on pin 13"* or *"Blink GPIO 2 three times"*, and a DeepSeek AI model running in the cloud converts your intent into precise hardware actions — no coding, no recompilation, no manual pin mapping.

**Key principle:** The LLM has **absolute control**. Every non-local command goes through the AI before touching hardware.

---

## Quick Start

### 1. Flash the Firmware

```bash
# Connect your ESP32 via USB, then:
cargo espflash flash --release --port /dev/ttyUSB0 --monitor

# Or if you already built:
espflash flash ~/esp32-agent-target/xtensa-esp32-espidf/release/esp32-agent \
    --port /dev/ttyUSB0 --monitor
```

> **Windows users:** Replace `/dev/ttyUSB0` with `COM3` (or your actual port).

### 2. First Boot — Provisioning Mode

On first boot (or when NVS is empty), the device enters **provisioning mode**:

```
=================================================
ESP32-Agent v1.0 — AI-Driven GPIO Controller
LLM has ABSOLUTE CONTROL — all HW via DeepSeek
=================================================
[MAIN] NVS config missing or incomplete: ...
[MAIN] Entering provisioning mode. Local commands only.
> 
```

**You MUST set these 5 values before the LLM will work:**

```
> config set wifi_ssid    MyHomeWiFi
OK: wifi_ssid=MyHomeWiFi
> config set wifi_pass    MyPassword
OK: wifi_pass=MyPassword
> config set tg_token     123456789:ABCdefGHIjklMNOpqrSTUvwxyz
OK: tg_token=123456789:ABCdefGHIjklMNOpqrSTUvwxyz
> config set ds_key       sk-xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
OK: ds_key=sk-xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
> config set tg_uid       123456789
OK: tg_uid=123456789
> reset
```

| Key | What it is | How to get it |
|-----|-----------|---------------|
| `wifi_ssid` | Your 2.4 GHz Wi-Fi name | Your router settings |
| `wifi_pass` | Your Wi-Fi password | Your router settings |
| `tg_token` | Telegram Bot API token | Message [@BotFather](https://t.me/botfather) on Telegram, create a bot, copy the token |
| `ds_key` | DeepSeek API key | Register at [platform.deepseek.com](https://platform.deepseek.com), generate a key |
| `tg_uid` | Your Telegram user ID | Message [@userinfobot](https://t.me/userinfobot) on Telegram, it replies with your ID |

After `reset`, the device connects to Wi-Fi and both the **Serial** and **Telegram** interfaces become active.

---

## Serial Monitor Interface (USB)

Open any serial terminal at **115200 baud** (or use the monitor built into `cargo espflash`).

### Local Commands (never touch hardware directly)

| Command | What it does |
|---------|-------------|
| `help` | Show available commands |
| `gpio list` | Print the capability table for all 23 GPIO pins |
| `config set <key> <value>` | Store a config key in flash (see provisioning table) |
| `reset` | Software reset the ESP32 |

### Natural-Language Commands (goes to DeepSeek AI)

**Anything that is NOT a local command is sent to the AI.** The AI decides which pin to use, what mode to set, and for how long.

```
> Turn on GPIO 13
[LLM] 'Turn on GPIO 13': Set GPIO 13 HIGH to turn on LED
[GPIO] GPIO13 -> 1

> Blink GPIO 2 three times
[LLM] 'Blink GPIO 2 three times': Blink GPIO 2 three times with 200ms on/off
[GPIO] GPIO2 -> 1
[GPIO] GPIO2 -> 0
...

> Set GPIO 18 PWM to 75%
[LLM] 'Set GPIO 18 PWM to 75%': Set GPIO 18 PWM duty to 192/255
[GPIO] GPIO18 PWM duty=192/255 (stub — LEDC not yet implemented)

> Read temperature on GPIO 34
[LLM] 'Read temperature on GPIO 34': Configure GPIO 34 as input
[GPIO] GPIO34 reading... (stub — input not yet implemented)
```

### Serial Examples

```
> gpio list
GPIO02 | out=true pwm=true in=true strap=true
GPIO04 | out=true pwm=true in=true strap=false
GPIO13 | out=true pwm=true in=true strap=false
GPIO34 | out=false pwm=false in=true strap=false
...

> Turn on the red LED
[LLM] 'Turn on the red LED': Set GPIO 13 HIGH to turn on red LED
[GPIO] GPIO13 -> 1

> Pulse GPIO 5 for 500ms
[LLM] 'Pulse GPIO 5 for 500ms': Set GPIO 5 HIGH for 500ms then LOW
[GPIO] GPIO5 -> 1
[GPIO]   auto-reset in 500ms (stub — timer not yet implemented)

> turn off all outputs
[LLM] 'turn off all outputs': Reset all output pins to LOW
[GPIO] GPIO13 -> 0
[GPIO] GPIO05 -> 0
...
```

---

## Telegram Bot Interface

Once provisioned, send messages to your bot on Telegram.

### Security

- Only the `allowed_user_id` stored in NVS can control the device.
- All other messages are **silently discarded**.
- Rate limit: **10 commands per minute**. Excess triggers: ⚠️ `Rate limit reached. Please wait.`

### Telegram Examples

| Your message | Bot reply | Hardware action |
|-------------|-----------|-----------------|
| `Turn on GPIO 13` | `Set GPIO 13 HIGH to turn on LED` | GPIO 13 → HIGH |
| `Blink GPIO 2 at 2 Hz` | `PWM square wave at 50% duty, 2 Hz on GPIO 2` | GPIO 2 PWM at 2 Hz |
| `Read temperature on GPIO 34` | `Configure GPIO 34 as input; read raw ADC` | GPIO 34 input mode |
| `Pulse GPIO 5 for 500ms` | `GPIO 5 HIGH for 500 ms, then auto-reset LOW` | GPIO 5 HIGH → 500ms → LOW |
| `Set GPIO 18 PWM to 75%` | `LEDC duty cycle = 192/255 on GPIO 18` | GPIO 18 PWM 75% |
| `Turn off all outputs` | `Reset all output pins to LOW` | All outputs → LOW |
| `What pins are available?` | LLM replies with capability summary | (none) |

---

## GPIO Capability Reference

| Pin | Output | PWM | Input | Strapping | Notes |
|-----|--------|-----|-------|-----------|-------|
| 2 | ✅ | ✅ | ✅ | ⚠️ | Boot mode pin — use with caution |
| 4 | ✅ | ✅ | ✅ | ❌ | |
| 5 | ✅ | ✅ | ✅ | ⚠️ | Boot mode pin — use with caution |
| 12 | ✅ | ✅ | ✅ | ⚠️ | Boot mode pin — use with caution |
| 13 | ✅ | ✅ | ✅ | ❌ | |
| 14 | ✅ | ✅ | ✅ | ❌ | |
| 15 | ✅ | ✅ | ✅ | ⚠️ | Boot mode pin — use with caution |
| 16–19 | ✅ | ✅ | ✅ | ❌ | |
| 21–23 | ✅ | ✅ | ✅ | ❌ | |
| 25–27 | ✅ | ✅ | ✅ | ❌ | |
| 32–33 | ✅ | ✅ | ✅ | ❌ | |
| 34–36, 39 | ❌ | ❌ | ✅ | ❌ | **Input-only** |

**Strapping pins** (0, 2, 15) are sampled at boot to configure chip mode. The executor logs a warning but still obeys the LLM if it uses them.

---

## How a Command Flows Through the System

```
You (Telegram / Serial)
    │
    │ "Blink GPIO 2 at 1 Hz for 10 seconds"
    ▼
telegram_task / serial_task
    │
    │ validate user (Telegram only)
    │ rate limit check
    ▼
DeepSeekClient::complete()
    │
    │ POST api.deepseek.com/chat/completions
    │ system prompt describes all 23 pins
    │ response_format: json_object
    ▼
JSON Plan: {"actions":[{"pin":2,"mode":"output","value":1}],"description":"..."}
    ▼
PinPlan validated against PIN_TABLE
    ▼
GpioRegistry::claim_output()
    ▼
Hardware pin toggles
    ▼
Confirmation sent back to user
```

---

## Troubleshooting

| Symptom | Cause | Fix |
|---------|-------|-----|
| `NVS config missing` on boot | First boot or flash erase | Run provisioning commands over Serial |
| `Wi-Fi failed` panic | Wrong SSID/password | `config set wifi_ssid` / `config set wifi_pass` then `reset` |
| `LLM error: ...` in Serial | No internet / bad API key | Check `ds_key` provisioning; ensure 2.4 GHz Wi-Fi |
| Telegram bot not responding | Wrong token / user ID | Re-provision `tg_token` and `tg_uid` |
| `Rate limit reached` | >10 Telegram commands/min | Wait 60 seconds |
| `pin cannot output` in logs | LLM chose input-only pin (34–36, 39) | Rephrase: *"Read from GPIO 34"* instead of *"Drive GPIO 34"* |
| `unknown pin` | LLM hallucinated invalid GPIO | Rephrase with a valid pin from the table |
| Serial shows `> ` but no echo | Terminal app issue | Use `cargo espflash --monitor` or press Enter |

---

## Re-Provisioning (Change Wi-Fi, Token, or Key)

You can update any config key at runtime over Serial without reflashing:

```
> config set wifi_ssid NewNetwork
OK: wifi_ssid=NewNetwork
> config set wifi_pass NewPassword
OK: wifi_pass=NewPassword
> config set ds_key sk-new-deepseek-key
OK: ds_key=sk-new-deepseek-key
> reset
```

---

## Flash & Monitor Cheat Sheet

```bash
# Build + flash + monitor in one step
cargo espflash flash --release --port /dev/ttyUSB0 --monitor

# Flash existing binary
espflash flash ~/esp32-agent-target/xtensa-esp32-espidf/release/esp32-agent \
    --port /dev/ttyUSB0 --monitor

# Check binary size breakdown
cargo size --release -- -A

# Erase all flash (factory reset)
espflash erase-flash --port /dev/ttyUSB0
```

---

## License

ESP32-Agent · MIT License · 2026
