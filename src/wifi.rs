//! Wi-Fi station driver — blocks until connected.

use esp_idf_hal::modem::Modem;
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    nvs::EspDefaultNvsPartition,
    wifi::{BlockingWifi, ClientConfiguration, Configuration, EspWifi},
};
use log::info;

/// Connect to the configured Wi-Fi network.
/// Returns a `BlockingWifi` handle that must stay alive for the
/// duration of the program.
pub fn connect(
    cfg: &crate::config::Config,
    sysloop: &EspSystemEventLoop,
    nvs: &EspDefaultNvsPartition,
    modem: Modem,
) -> anyhow::Result<BlockingWifi<EspWifi<'static>>> {
    info!("[WiFi] Initialising Wi-Fi station...");

    let wifi_driver = EspWifi::new(modem, sysloop.clone(), Some(nvs.clone()))?;
    let mut wifi = BlockingWifi::wrap(wifi_driver, sysloop.clone())?;

    wifi.set_configuration(&Configuration::Client(ClientConfiguration {
        ssid: cfg.wifi_ssid.as_str().try_into().unwrap(),
        password: cfg.wifi_pass.as_str().try_into().unwrap(),
        ..Default::default()
    }))?;

    info!("[WiFi] Starting...");
    wifi.start()?;

    info!("[WiFi] Connecting to '{}'...", cfg.wifi_ssid);
    wifi.connect()?;

    info!("[WiFi] Waiting for IP...");
    wifi.wait_netif_up()?;

    let ip_info = wifi.wifi().sta_netif().get_ip_info()?;
    info!("[WiFi] Connected — IP: {:?}", ip_info);

    Ok(wifi)
}
