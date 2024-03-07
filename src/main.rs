use esp_idf_svc::hal::{
    delay::{Delay, FreeRtos},
    gpio::PinDriver,
    peripherals::Peripherals,
};

use loadcell::{hx711::HX711, LoadCell};

fn main() {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();
    let dt = PinDriver::input(peripherals.pins.gpio2).unwrap();
    let sck = PinDriver::output(peripherals.pins.gpio3).unwrap();
    let delay = Delay::new_default();

    let mut load_sensor = HX711::new(sck, dt, delay);

    load_sensor.tare(16);
    load_sensor.set_scale(1.0);

    loop {
        if load_sensor.is_ready() {
            let reading = load_sensor.read_scaled().unwrap();
            // let reading = load_sensor.read().unwrap(); // Use this one to calibrate the load cell
            log::info!("Weight: {:.0} g", reading);
            // log::info!("Weight: {} g", reading); // Use this to get all the decimals
        }

        FreeRtos::delay_ms(1000u32);
    }
}
