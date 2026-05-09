//! NVS-backed persistent configuration.

use esp_idf_svc::nvs::{EspDefaultNvsPartition, EspNvs, NvsDefault};
use heapless::String;
use log::info;

const NVS_NAMESPACE: &str = "claw_cfg";

/// Runtime configuration loaded once at boot.
pub struct Config {
    pub wifi_ssid: String<64>,
    pub wifi_pass: String<64>,
    pub telegram_token: String<128>,
    pub deepseek_key: String<128>,
    pub allowed_user_id: i64,
}

impl Config {
    /// Load from NVS. If keys are missing, returns an error so the
    /// caller can fall back to provisioning mode.
    pub fn load(nvs_part: &EspDefaultNvsPartition) -> anyhow::Result<Self> {
        let nvs = EspNvs::new(nvs_part.clone(), NVS_NAMESPACE, true)?;

        let cfg = Self {
            wifi_ssid: nvs_str(&nvs, "wifi_ssid")?,
            wifi_pass: nvs_str(&nvs, "wifi_pass")?,
            telegram_token: nvs_str(&nvs, "tg_token")?,
            deepseek_key: nvs_str(&nvs, "ds_key")?,
            allowed_user_id: nvs_i64(&nvs, "tg_uid")?,
        };

        info!("Config loaded from NVS");
        Ok(cfg)
    }

    /// Store a single key-value pair in NVS.
    pub fn set(
        nvs_part: &EspDefaultNvsPartition,
        key: &str,
        value: &str,
    ) -> anyhow::Result<()> {
        let mut nvs = EspNvs::new(nvs_part.clone(), NVS_NAMESPACE, true)?;
        nvs.set_str(key, value)?;
        info!("NVS set: {} = {}", key, value);
        Ok(())
    }
}

// ------------------------------------------------------------------
// Helpers
// ------------------------------------------------------------------

fn nvs_str<const N: usize>(nvs: &EspNvs<NvsDefault>, key: &str) -> anyhow::Result<String<N>> {
    let mut buf = [0u8; N];
    let data = nvs.get_raw(key, &mut buf)?
        .ok_or_else(|| anyhow::anyhow!("NVS key '{}' is empty", key))?;
    let mut s = String::new();
    s.push_str(core::str::from_utf8(data)?).map_err(|_| anyhow::anyhow!("string overflow"))?;
    Ok(s)
}

fn nvs_i64(nvs: &EspNvs<NvsDefault>, key: &str) -> anyhow::Result<i64> {
    let mut buf = [0u8; 8];
    let data = nvs.get_raw(key, &mut buf)?
        .ok_or_else(|| anyhow::anyhow!("NVS key '{}' is empty", key))?;
    if data.len() != 8 {
        return Err(anyhow::anyhow!("NVS key '{}' has wrong length", key));
    }
    let bytes: [u8; 8] = data.try_into()?;
    Ok(i64::from_le_bytes(bytes))
}
