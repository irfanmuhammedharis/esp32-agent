# Graph Report - esp32 agent  (2026-05-09)

## Corpus Check
- 19 files · ~15,281 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 226 nodes · 232 edges · 24 communities (17 shown, 7 thin omitted)
- Extraction: 98% EXTRACTED · 2% INFERRED · 0% AMBIGUOUS · INFERRED: 4 edges (avg confidence: 0.8)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `6bdf05a2`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- [[_COMMUNITY_Community 0|Community 0]]
- [[_COMMUNITY_Community 1|Community 1]]
- [[_COMMUNITY_Community 2|Community 2]]
- [[_COMMUNITY_Community 3|Community 3]]
- [[_COMMUNITY_Community 4|Community 4]]
- [[_COMMUNITY_Community 5|Community 5]]
- [[_COMMUNITY_Community 6|Community 6]]
- [[_COMMUNITY_Community 7|Community 7]]
- [[_COMMUNITY_Community 8|Community 8]]
- [[_COMMUNITY_Community 9|Community 9]]
- [[_COMMUNITY_Community 10|Community 10]]
- [[_COMMUNITY_Community 11|Community 11]]
- [[_COMMUNITY_Community 12|Community 12]]
- [[_COMMUNITY_Community 13|Community 13]]
- [[_COMMUNITY_Community 14|Community 14]]
- [[_COMMUNITY_Community 15|Community 15]]
- [[_COMMUNITY_Community 16|Community 16]]
- [[_COMMUNITY_Community 17|Community 17]]
- [[_COMMUNITY_Community 18|Community 18]]
- [[_COMMUNITY_Community 19|Community 19]]
- [[_COMMUNITY_Community 20|Community 20]]
- [[_COMMUNITY_Community 22|Community 22]]
- [[_COMMUNITY_Community 23|Community 23]]

## God Nodes (most connected - your core abstractions)
1. `ESP32-Agent` - 20 edges
2. `ESP32-Agent Implementation Plan — RTOS-Centric` - 14 edges
3. `GpioRegistry` - 13 edges
4. `ESP32-Agent User Guide` - 12 edges
5. `6. Core Implementation` - 9 edges
6. `ESP32-Agent` - 8 edges
7. `Vm` - 7 edges
8. `10. Security Considerations` - 6 edges
9. `4. Toolchain Setup` - 5 edges
10. `9. Memory Management Strategy` - 5 edges

## Surprising Connections (you probably didn't know these)
- `main()` --calls--> `connect()`  [INFERRED]
  src/main.rs → src/wifi.rs

## Communities (24 total, 7 thin omitted)

### Community 0 - "Community 0"
Cohesion: 0.08
Nodes (23): 14. Troubleshooting, 15. Roadmap, 16. Glossary, 17. References & Further Reading, 1.1 Design Pillars, 1.2 High-Level Architecture, 1. Project Overview, 2.1 Hardware (+15 more)

### Community 1 - "Community 1"
Cohesion: 0.09
Nodes (21): 1. Flash the Firmware, 2. First Boot — Provisioning Mode, AI-Driven GPIO Controller for ESP32, code:bash (# Connect your ESP32 via USB, then:), code:block2 (=================================================), code:block3 (> config set wifi_ssid    MyHomeWiFi), code:block6 (You (Telegram / Serial)), code:block7 (> config set wifi_ssid NewNetwork) (+13 more)

### Community 2 - "Community 2"
Cohesion: 0.13
Nodes (5): GpioRegistry, lookup(), PinSpec, main(), connect()

### Community 3 - "Community 3"
Cohesion: 0.12
Nodes (17): 6.1 main.rs — Entry Point & Task Orchestration, 6.2 config.rs — NVS-Backed Secure Storage, 6.3 wifi.rs — Wi-Fi Station Driver, 6.4 telegram/api.rs — Telegram Bot Polling, 6.5 llm/deepseek.rs — LLM API Client, 6.6 llm/plan.rs — Structured Plan Types, 6.7 gpio/registry.rs — Pin Capability Table, 6.8 gpio/executor.rs — Plan Execution Engine (+9 more)

### Community 4 - "Community 4"
Cohesion: 0.12
Nodes (16): 1. Flash, 2. Provision (first boot only), 3. Control, Architecture, code:bash (cargo espflash flash --release --port /dev/ttyUSB0 --monitor), code:block2 (> config set wifi_ssid  MyHomeWiFi), code:block3 (User (Telegram / Serial)), code:block4 (├── src/) (+8 more)

### Community 5 - "Community 5"
Cohesion: 0.13
Nodes (14): Derived from document.md v1.0 · FreeRTOS-First Architecture, Design Philosophy, ESP32-Agent Implementation Plan — RTOS-Centric, Inter-Task Communication Map, Phase 1: Toolchain & RTOS-Aware Project Bootstrap, Phase 2: RTOS Task Foundation — Config, Wi-Fi, Serial, Phase 3: GPIO Subsystem — Queue-Driven Executor, Phase 4: LLM Integration — Synchronous HTTP in Dedicated Task (+6 more)

### Community 6 - "Community 6"
Cohesion: 0.27
Nodes (7): Config, nvs_i64(), nvs_str(), handle_line(), parse(), print_prompt(), run()

### Community 8 - "Community 8"
Cohesion: 0.22
Nodes (9): 4.1 Install Rust with Espressif Support, 4.2 Install ESP-IDF v5.1, 4.3 Install cargo-espflash, 4.4 Project Bootstrap, 4. Toolchain Setup, code:bash (# Install rustup), code:bash (git clone --recursive https://github.com/espressif/esp-idf.g), code:bash (cargo install cargo-espflash) (+1 more)

### Community 9 - "Community 9"
Cohesion: 0.22
Nodes (9): 10.1 Telegram User Whitelist, 10.2 TLS Certificate Validation, 10.3 LLM Plan Sandboxing, 10.4 API Key Storage, 10.5 Rate Limiting, 10. Security Considerations, code:rust (if update.message.from.id != cfg.allowed_user_id {), code:rust (EspHttpConnection::new(&HttpConfig {) (+1 more)

### Community 10 - "Community 10"
Cohesion: 0.22
Nodes (9): 12.1 First-Time Credential Provisioning, 12.2 Build & Flash, 12.3 Partition Table, 12.4 OTA Updates (Optional), 12. Build, Flash & Monitor, code:bash (# Provision via Serial CLI after first flash), code:bash (# Debug build (faster compile, larger binary)), code:csv (# partitions.csv) (+1 more)

### Community 11 - "Community 11"
Cohesion: 0.22
Nodes (9): 13.1 Adding a New LLM Backend, 13.2 Adding MQTT Transport, 13.3 Adding I2C / SPI Peripherals, 13.4 Adding Voice Commands, 13. Extending ESP32-Agent, code:rust (// src/llm/mod.rs), code:rust (// Pseudo-code for MQTT integration), code:rust (// Append to SYSTEM_PROMPT dynamically at runtime) (+1 more)

### Community 12 - "Community 12"
Cohesion: 0.25
Nodes (8): 9.1 Heap Usage Budget, 9.2 Rust Ownership for Pin Safety, 9.3 Stack-Only Strings with `heapless`, 9.4 Static Dispatch over Dynamic Dispatch, 9. Memory Management Strategy, code:rust (// Zero-cost abstraction — ownership enforced at compile tim), code:rust (use heapless::String;), code:rust (// Preferred: zero-cost static dispatch)

### Community 13 - "Community 13"
Cohesion: 0.29
Nodes (6): ApiResponse, Chat, GetUpdatesResponse, Message, Update, User

### Community 14 - "Community 14"
Cohesion: 0.33
Nodes (6): 11.1 Telegram Command Examples, 11.2 Example LLM JSON Plan, 11.3 Multi-Pin Example, 11. Usage Examples, code:json (// User: "Blink the blue LED on GPIO 2 three times, 200ms ea), code:json (// User: "Turn on the fan (GPIO 26) and set the indicator LE)

### Community 15 - "Community 15"
Cohesion: 0.33
Nodes (6): code:block4 (> Turn on GPIO 13), code:block5 (> gpio list), Local Commands (never touch hardware directly), Natural-Language Commands (goes to DeepSeek AI), Serial Examples, Serial Monitor Interface (USB)

### Community 17 - "Community 17"
Cohesion: 0.4
Nodes (5): 5.1 Cargo.toml — Key Dependencies, 5.2 .cargo/config.toml, 5. Configuration, code:toml ([package]), code:toml ([build])

## Knowledge Gaps
- **98 isolated node(s):** `Command`, `Update`, `Message`, `User`, `Chat` (+93 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **7 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `ESP32-Agent` connect `Community 0` to `Community 3`, `Community 8`, `Community 9`, `Community 10`, `Community 11`, `Community 12`, `Community 14`, `Community 17`?**
  _High betweenness centrality (0.163) - this node is a cross-community bridge._
- **Why does `6. Core Implementation` connect `Community 3` to `Community 0`?**
  _High betweenness centrality (0.055) - this node is a cross-community bridge._
- **Why does `10. Security Considerations` connect `Community 9` to `Community 0`?**
  _High betweenness centrality (0.029) - this node is a cross-community bridge._
- **What connects `Command`, `Update`, `Message` to the rest of the system?**
  _98 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Community 0` be split into smaller, more focused modules?**
  _Cohesion score 0.08 - nodes in this community are weakly interconnected._
- **Should `Community 1` be split into smaller, more focused modules?**
  _Cohesion score 0.09 - nodes in this community are weakly interconnected._
- **Should `Community 2` be split into smaller, more focused modules?**
  _Cohesion score 0.13 - nodes in this community are weakly interconnected._