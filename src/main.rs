//! ESP32-Agent entry point.

extern crate esp_idf_sys;

use esp_idf_hal::prelude::*;
use esp_idf_hal::uart::UartDriver;
use esp_idf_hal::uart::config::Config as UartConfig;
use esp_idf_hal::units::Hertz;
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    nvs::EspDefaultNvsPartition,
};
use log::info;
use std::sync::mpsc::sync_channel;

mod config;
mod gpio;
mod llm;
mod program;
mod rtos;
mod serial;
mod telegram;
mod wifi;

fn main() {
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    info!("=================================================");
    info!("ESP32-agent v1.0 — AI-Driven GPIO Controller");
    info!("LLM has ABSOLUTE CONTROL — projects via DeepSeek");
    info!("=================================================");

    let mut peripherals = Peripherals::take().unwrap();
    let sysloop = EspSystemEventLoop::take().unwrap();
    let nvs = EspDefaultNvsPartition::take().unwrap();

    // Create GPIO registry first (extracts pins safely from Peripherals)
    let gpio_registry = match gpio::GpioRegistry::from_peripherals(&mut peripherals) {
        Ok(r) => r,
        Err(e) => {
            log::error!("[MAIN] GPIO registry init failed: {:?}", e);
            return;
        }
    };

    // Create UART driver for serial task
    let uart = match UartDriver::new(
        peripherals.uart0,
        peripherals.pins.gpio1,
        peripherals.pins.gpio3,
        None::<esp_idf_hal::gpio::AnyIOPin>,
        None::<esp_idf_hal::gpio::AnyIOPin>,
        &UartConfig::default().baudrate(Hertz(115_200)),
    ) {
        Ok(u) => u,
        Err(e) => {
            log::error!("[MAIN] UART init failed: {:?}", e);
            return;
        }
    };

    // Load persisted config
    let cfg = match config::Config::load(&nvs) {
        Ok(c) => c,
        Err(e) => {
            info!("[MAIN] NVS config missing or incomplete: {:?}", e);
            info!("[MAIN] Entering provisioning mode. Local commands only.");
            let (tx, rx) = sync_channel::<llm::plan::Project>(rtos::PLAN_QUEUE_DEPTH);
            let empty_key = heapless::String::<128>::new();
            let nvs_serial = nvs.clone();
            std::thread::Builder::new()
                .name("serial".into())
                .stack_size(4096)
                .spawn(move || serial::run(uart, tx, empty_key, nvs_serial))
                .unwrap();
            // In provisioning mode the program task just waits
            program::run(rx, gpio_registry);
            return;
        }
    };

    // Bring up Wi-Fi
    let _wifi = wifi::connect(&cfg, &sysloop, &nvs, peripherals.modem)
        .expect("Wi-Fi failed");

    // Shared project queue (capacity = 2)
    let (tx, rx) = sync_channel::<llm::plan::Project>(rtos::PLAN_QUEUE_DEPTH);

    // Clone config for task moves
    let cfg_telegram = config::Config {
        wifi_ssid: cfg.wifi_ssid.clone(),
        wifi_pass: cfg.wifi_pass.clone(),
        telegram_token: cfg.telegram_token.clone(),
        deepseek_key: cfg.deepseek_key.clone(),
        allowed_user_id: cfg.allowed_user_id,
    };
    let nvs_telegram = nvs.clone();
    let tx_telegram = tx.clone();
    let ds_key_serial = cfg.deepseek_key.clone();
    let nvs_serial = nvs.clone();

    // Spawn Telegram task
    std::thread::Builder::new()
        .name("telegram".into())
        .stack_size(8192)
        .spawn(move || telegram::run(cfg_telegram, tx_telegram, nvs_telegram))
        .unwrap();

    // Spawn Serial task (with its own LLM client for absolute control)
    std::thread::Builder::new()
        .name("serial".into())
        .stack_size(4096)
        .spawn(move || serial::run(uart, tx, ds_key_serial, nvs_serial))
        .unwrap();

    // Program task runs on the root task (never returns)
    program::run(rx, gpio_registry);
}
