//! DeepSeek Chat Completions client.

use crate::llm::plan::Project;
use crate::llm::LlmClient;
use esp_idf_svc::http::client::{Configuration as HttpConfig, EspHttpConnection};
use embedded_svc::http::client::Client as HttpClient;
use embedded_svc::http::Method;
use heapless::String;
use log::{error, info};
use serde_json::json;

const API_HOST: &str = "api.deepseek.com";
const SYSTEM_PROMPT: &str = r#"
You are an ESP32 hardware agent. The user describes an electronics project in natural language.
Your job is to produce TWO things:
1. A wiring guide so the user knows how to connect components.
2. A program that the ESP32 will run continuously.

Respond ONLY with a valid JSON object matching this schema:
{
  "wiring_instructions": "<plain-English step-by-step wiring guide>",
  "description": "<one-sentence summary>",
  "setup": [
    {"config_output": {"pin": 13}},
    {"config_input": {"pin": 34}}
  ],
  "loop_body": [
    {"read_adc": {"pin": 34, "var": "moisture"}},
    {"jump_if_lt": {"var": "moisture", "value": 128, "label": "dry"}},
    {"set_low": {"pin": 13}},
    {"jump": {"label": "end"}},
    {"label": {"name": "dry"}},
    {"set_high": {"pin": 13}},
    {"label": {"name": "end"}},
    {"delay": {"ms": 1000}}
  ],
  "interval_ms": 1000
}

INSTRUCTION SET (use snake_case keys):
- config_output / config_input / config_pwm : configure pin mode
- set_high / set_low : digital write
- set_pwm : {"pin": N, "duty": 0..255}
- read_adc : {"pin": N, "var": "name"} — reads 0-4095, stores in variable
- read_digital : {"pin": N, "var": "name"} — reads 0 or 1
- delay : {"ms": N}
- set_var : {"var": "name", "value": N}
- label : {"name": "label_name"} — jump target, no-op
- jump : {"label": "label_name"} — unconditional
- jump_if_lt / jump_if_gt / jump_if_eq : conditional jumps

Available GPIO: 2,4,5,12,13,14,15,16,17,18,19,21,22,23,25,26,27,32,33,34,35,36,39.
PWM-capable:    2,4,5,12,13,14,15,16,17,18,19,21,22,23,25,26,27,32,33.
Input-only (no output): 34,35,36,39.
Do not use strapping pins (0,2,15) for persistent output.

EXAMPLE — Soil Moisture Alert:
"Create an alert using soil moisture sensor. When soil is dry, turn on red LED."
→ wiring_instructions: "1) Connect soil sensor VCC to ESP32 3.3V. 2) Connect sensor GND to ESP32 GND. 3) Connect sensor AO to GPIO34. 4) Connect red LED anode to GPIO13 through a 220Ω resistor. 5) Connect LED cathode to GND."
→ setup: [config_output(13), config_input(34)]
→ loop_body: [read_adc(34, "moisture"), jump_if_lt("moisture", 128, "dry"), set_low(13), jump("end"), label("dry"), set_high(13), label("end"), delay(1000)]
→ interval_ms: 1000
"#;

pub struct DeepSeekClient {
    api_key: String<128>,
}

impl DeepSeekClient {
    pub fn new(api_key: &str) -> anyhow::Result<Self> {
        let mut key = String::<128>::new();
        key.push_str(api_key).map_err(|_| anyhow::anyhow!("API key too long"))?;
        Ok(Self { api_key: key })
    }
}

impl LlmClient for DeepSeekClient {
    fn complete(&self, instruction: &str) -> anyhow::Result<Project> {
        info!("[LLM] Sending to DeepSeek...");

        let body = json!({
            "model": "deepseek-chat",
            "messages": [
                { "role": "system", "content": SYSTEM_PROMPT },
                { "role": "user",   "content": instruction }
            ],
            "response_format": { "type": "json_object" },
            "temperature": 0.1,
            "max_tokens": 1024
        });

        let body_str = body.to_string();
        let authorization = format!("Bearer {}", self.api_key.as_str());

        let url = format!("https://{}/chat/completions", API_HOST);

        let mut client = HttpClient::wrap(
            EspHttpConnection::new(&HttpConfig {
                use_global_ca_store: true,
                crt_bundle_attach: Some(esp_idf_sys::esp_crt_bundle_attach),
                ..Default::default()
            })?,
        );

        let headers = [
            ("Content-Type", "application/json"),
            ("Authorization", authorization.as_str()),
        ];

        let mut request = client.request(Method::Post, &url, &headers)?;
        request.write(body_str.as_bytes())?;
        let mut response = request.submit()?;

        let status = response.status();
        if status != 200 {
            return Err(anyhow::anyhow!("DeepSeek HTTP {}", status));
        }

        let mut buf = [0u8; 4096];
        let mut read = 0usize;
        loop {
            let n = response.read(&mut buf[read..])?;
            if n == 0 {
                break;
            }
            read += n;
            if read >= buf.len() {
                break;
            }
        }

        let text = core::str::from_utf8(&buf[..read])?;
        info!("[LLM] Raw response length: {}", read);

        // DeepSeek nests JSON inside choices[0].message.content
        let parsed: serde_json::Value = serde_json::from_str(text)?;
        let content = parsed["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing content in LLM response"))?;

        info!("[LLM] Project JSON: {}", content);

        let project: Project = serde_json::from_str(content)
            .map_err(|e| {
                error!("JSON parse error: {:?}", e);
                anyhow::anyhow!("Invalid project JSON: {:?}", e)
            })?;

        Ok(project)
    }
}
