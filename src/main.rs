use crate::scale::Scale;
use esp_idf_svc::hal::{delay::FreeRtos, peripherals::Peripherals};

mod critical_section;
mod scale;

const LOAD_SENSOR_SCALING: f32 = 0.0027;

fn main() {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();
    let dt = peripherals.pins.gpio2;
    let sck = peripherals.pins.gpio3;

    let mut scale = Scale::new(sck, dt, LOAD_SENSOR_SCALING).unwrap();
    scale.wait_stable();
    scale.tare(32);

    loop {
        if scale.is_ready() {
            let rounded_reading = scale.read_rounded().unwrap();

            log::info!("Weight: {} g", rounded_reading);
        }

        FreeRtos::delay_ms(1000u32);
    }
}
