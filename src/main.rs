//! Example of using blocking wifi.
//!
//! Add your own ssid and password

use embedded_svc::{
    http::client::Client as HttpClient,
    wifi::{AuthMethod, ClientConfiguration, Configuration},
};
use esp_idf_hal::prelude::Peripherals;
use esp_idf_hal::sys::esp_wifi_set_max_tx_power;
use esp_idf_svc::http::client::EspHttpConnection;
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

    let mut client = HttpClient::wrap(EspHttpConnection::new(&Default::default())?);

    // let mut client = HttpClient::wrap(EspHttpConnection::new(&Configuration {
    //     crt_bundle_attach: Some(esp_idf_sys::esp_crt_bundle_attach),

    //     ..Default::default()
    // })?);

    // GET
    get_request(&mut client)?;

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
