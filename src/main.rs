use std::collections::VecDeque;

use esp_idf_svc::hal::{
    delay::{Delay, Ets, FreeRtos},
    gpio::{Gpio2, Gpio3, Input, Output, PinDriver},
    peripherals::Peripherals,
};

use loadcell::{hx711::HX711, LoadCell};

mod critical_section;

const LOADCELL_STABLE_READINGS: usize = 10; // Number of readings to consider for stability
const LOADCELL_LOOP_DELAY_US: u32 = 10000; // Delay between readings in microseconds
                                           // const LOADCELL_READY_DELAY_US: u32 = 1000;
fn wait_stable(
    load_sensor: &mut HX711<PinDriver<'_, Gpio3, Output>, PinDriver<'_, Gpio2, Input>, Delay>,
) {
    let mut readings: VecDeque<f32> = VecDeque::with_capacity(LOADCELL_STABLE_READINGS);
    loop {
        while !load_sensor.is_ready() {
            FreeRtos::delay_ms(10);
        }
        let reading = load_sensor.read_scaled().expect("Failed to read scale");
        log::info!("Waiting for stable weight: {:.4}", reading);
        if readings.len() == LOADCELL_STABLE_READINGS {
            readings.pop_front();
        }
        readings.push_back(reading);
        if readings.len() == LOADCELL_STABLE_READINGS
            && readings.iter().all(|&x| (x - reading).abs() < 0.2)
        {
            break;
        }
        Ets::delay_us(LOADCELL_LOOP_DELAY_US); // Adjust delay as necessary
    }
}

fn main() {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();
    let dt = PinDriver::input(peripherals.pins.gpio2).unwrap();
    let sck = PinDriver::output(peripherals.pins.gpio3).unwrap();
    let delay = Delay::new_default();

    let mut load_sensor = HX711::new(sck, dt, delay);

    // load_sensor.tare(16);
    // load_sensor.set_scale(1.0);
    // load_sensor.set_scale(0.0043);
    load_sensor.set_scale(0.0027);
    wait_stable(&mut load_sensor);
    load_sensor.tare(32);

    loop {
        if load_sensor.is_ready() {
            let reading = load_sensor.read_scaled().unwrap();
            // let reading = load_sensor.read().unwrap(); // Use this one to calibrate the load cell
            // log::info!("Weight: {:.0} g", reading);
            let rounded_reading = if reading.round() == -0f32 {
                0
            } else {
                reading.round() as i32
            };
            log::info!("Weight: {} g", rounded_reading); // Use this to get all the decimals
        }

        FreeRtos::delay_ms(1000u32);
    }
}
