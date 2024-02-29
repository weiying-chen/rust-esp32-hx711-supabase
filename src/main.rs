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
    load_sensor.set_scale(0.0028);

    loop {
        if load_sensor.is_ready() {
            let reading = load_sensor.read_scaled();
            log::info!("Last Reading = {:?}", reading);
        }

        FreeRtos::delay_ms(100u32);
    }
}
