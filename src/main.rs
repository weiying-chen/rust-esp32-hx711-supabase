//! Example of using blocking wifi.
//!
//! Add your own ssid and password

use embedded_svc::{
    http::client::Client as HttpClient,
    io::Write,
    wifi::{AuthMethod, ClientConfiguration, Configuration},
};
use esp_idf_hal::prelude::Peripherals;
use esp_idf_hal::sys::esp_wifi_set_max_tx_power;
use esp_idf_svc::http::client::{Configuration as HttpConfiguration, EspHttpConnection};
use esp_idf_svc::log::EspLogger;
use esp_idf_svc::wifi::{BlockingWifi, EspWifi};
use esp_idf_svc::{eventloop::EspSystemEventLoop, nvs::EspDefaultNvsPartition};
// use esp_idf_sys::{self as _}; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported
use log::info;

const SSID: &str = env!("WIFI_SSID");
const PASSWORD: &str = env!("WIFI_PASS");

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

    // GET
    get_request(&mut client)?;

    // GET
    post_request(&mut client)?;

    // let ip_info = wifi.wifi().sta_netif().get_ip_info()?;

    // info!("Wifi DHCP info: {:?}", ip_info);

    // info!("Shutting down in 5s...");

    std::thread::sleep(core::time::Duration::from_secs(5));

    Ok(())
}

fn get_request(client: &mut HttpClient<EspHttpConnection>) -> anyhow::Result<()> {
    let url = "https://httpbin.org/get";
    let request = client.get(url)?;

    info!("-> GET {}", url);

    let response = request.submit()?;

    info!("<- {}", response.status());

    if let Some(content_length) = response.header("Content-Length") {
        info!("Content-Length: {}", content_length);
    }

    if let Some(date) = response.header("Date") {
        info!("Date: {}", date);
    }

    std::thread::sleep(core::time::Duration::from_secs(5));

    Ok(())
}

fn post_request(client: &mut HttpClient<EspHttpConnection>) -> anyhow::Result<()> {
    let supabase_url = "https://pratqgdulutgohggfwfo.supabase.co/rest/v1/messages";
    let supabase_key = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6InByYXRxZ2R1bHV0Z29oZ2dmd2ZvIiwicm9sZSI6ImFub24iLCJpYXQiOjE3MTA0Mjc1NzgsImV4cCI6MjAyNjAwMzU3OH0.miKfZwWualZGbxDZ7KQpvaOK_Rxw6mbQ_EpiPMKi318";
    let payload = br#"{"content": "This is a message from ESP32"}"#;
    let content_length_header = format!("{}", payload.len());

    let headers = [
        ("apikey", supabase_key),
        ("Authorization", &format!("Bearer {}", supabase_key)),
        ("Content-Type", "application/json"),
        // ("Prefer", "return=representation"),
        ("Content-Length", &content_length_header),
    ];

    let mut request = client.post(supabase_url, &headers)?;

    request.write_all(payload)?;
    request.flush()?;
    info!("-> POST {}", supabase_url);

    let response = request.submit()?;
    let status = response.status();

    info!("<- {}", status);

    // let mut buf = [0u8; 1024];
    // let bytes_read = io::try_read_full(&mut response, &mut buf).map_err(|e| e.0)?;
    // info!("Read {} bytes", bytes_read);
    // match std::str::from_utf8(&buf[0..bytes_read]) {
    //     Ok(body_string) => info!(
    //         "Response body (truncated to {} bytes): {:?}",
    //         buf.len(),
    //         body_string
    //     ),
    //     Err(e) => error!("Error decoding response body: {}", e),
    // };

    // Drain any remaining bytes in the response to complete reading
    // while response.read(&mut buf)? > 0 {}

    Ok(())
}

fn connect_wifi(wifi: &mut BlockingWifi<EspWifi<'static>>) -> anyhow::Result<()> {
    let wifi_configuration: Configuration = Configuration::Client(ClientConfiguration {
        ssid: SSID.try_into().unwrap(),
        bssid: None,
        auth_method: AuthMethod::WPA2Personal,
        password: PASSWORD.try_into().unwrap(),
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
