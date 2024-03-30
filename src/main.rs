use embedded_svc::{
    http::client::Client as HttpClient,
    io::Write,
    utils::io,
    wifi::{AuthMethod, ClientConfiguration, Configuration},
};
use esp_idf_hal::prelude::Peripherals;
use esp_idf_hal::sys::esp_wifi_set_max_tx_power;
use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::http::client::{Configuration as HttpConfiguration, EspHttpConnection};
use esp_idf_svc::log::EspLogger;
use esp_idf_svc::wifi::{BlockingWifi, EspWifi};
use esp_idf_svc::{eventloop::EspSystemEventLoop, nvs::EspDefaultNvsPartition};
use esp_idf_sys::{self as _}; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported
use log::{error, info};

use crate::scale::Scale;
mod critical_section;
mod scale;

const WIFI_SSID: &str = env!("WIFI_SSID");
const WIFI_PASSWORD: &str = env!("WIFI_PASS");
const SUPABASE_KEY: &str = env!("SUPABASE_KEY");
const SUPABASE_URL: &str = env!("SUPABASE_URL");
const LOAD_SENSOR_SCALING: f32 = 0.0027;

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();
    let sys_loop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;

    let mut wifi = BlockingWifi::wrap(
        EspWifi::new(peripherals.modem, sys_loop.clone(), Some(nvs))?,
        sys_loop,
    )?;

    connect_wifi(&mut wifi)?;

    let config = &HttpConfiguration {
        buffer_size: Some(1024),
        buffer_size_tx: Some(1024),
        crt_bundle_attach: Some(esp_idf_svc::sys::esp_crt_bundle_attach),
        ..Default::default()
    };

    let mut client = HttpClient::wrap(EspHttpConnection::new(&config)?);
    let dt = peripherals.pins.gpio2;
    let sck = peripherals.pins.gpio3;
    let mut scale = Scale::new(sck, dt, LOAD_SENSOR_SCALING).unwrap();

    scale.tare(32);

    let mut iterations = 0;

    loop {
        if scale.is_ready() {
            log::info!("Iteration {}", iterations);

            let rounded_reading = scale.read_rounded().unwrap();
            let message = format!("This is a message from ESP32: {} g", rounded_reading);

            log::info!("{}", message);

            let payload = serde_json::json!({
                "content": message
            });

            let payload_str = serde_json::to_string(&payload).unwrap();
            let payload_bytes = payload_str.as_bytes();

            post_request(&mut client, payload_bytes)?;
        }

        FreeRtos::delay_ms(10000u32);

        iterations += 1;

        if iterations >= 4 {
            break;
        }
    }

    info!("Shutting down in 5s...");

    std::thread::sleep(core::time::Duration::from_secs(5));

    Ok(())
}

fn post_request(client: &mut HttpClient<EspHttpConnection>, payload: &[u8]) -> anyhow::Result<()> {
    let content_length_header = format!("{}", payload.len());

    let headers = [
        ("apikey", SUPABASE_KEY),
        ("Authorization", &format!("Bearer {}", SUPABASE_KEY)),
        ("Content-Type", "application/json"),
        ("Prefer", "return=representation"),
        ("Content-Length", &content_length_header),
    ];

    let mut request = client.post(SUPABASE_URL, &headers)?;

    request.write_all(payload)?;
    request.flush()?;

    info!("-> POST {}", SUPABASE_URL);

    let mut response = request.submit()?;
    let status = response.status();

    info!("<- {}", status);

    let mut buf = [0u8; 1024];
    let bytes_read = io::try_read_full(&mut response, &mut buf).map_err(|e| e.0)?;

    info!("Read {} bytes", bytes_read);

    match std::str::from_utf8(&buf[0..bytes_read]) {
        Ok(body_string) => info!(
            "Response body (truncated to {} bytes): {:?}",
            buf.len(),
            body_string
        ),
        Err(e) => error!("Error decoding response body: {}", e),
    };

    while response.read(&mut buf)? > 0 {}

    Ok(())
}

fn connect_wifi(wifi: &mut BlockingWifi<EspWifi<'static>>) -> anyhow::Result<()> {
    let wifi_configuration: Configuration = Configuration::Client(ClientConfiguration {
        ssid: WIFI_SSID.try_into().unwrap(),
        bssid: None,
        auth_method: AuthMethod::WPA2Personal,
        password: WIFI_PASSWORD.try_into().unwrap(),
        channel: None,
    });

    wifi.set_configuration(&wifi_configuration)?;
    wifi.start()?;
    info!("Wifi started");

    unsafe { esp_wifi_set_max_tx_power(34) };

    wifi.connect()?;
    info!("Wifi connected");
    wifi.wait_netif_up()?;
    info!("Wifi netif up");

    Ok(())
}
