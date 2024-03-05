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
            let reading = load_sensor.read_scaled().unwrap();
            log::info!("Weight: {:.0} g", reading);
        }

        FreeRtos::delay_ms(1000u32);
    }
}

// use critical_section::set_impl;
// use esp_idf_svc::hal::interrupt::{IsrCriticalSection, IsrCriticalSectionGuard};
// use esp_idf_svc::hal::{delay, gpio::PinDriver, peripherals::Peripherals};
// use loadcell::{hx711::HX711, LoadCell};
// use std::sync::Mutex;

// static CS: IsrCriticalSection = IsrCriticalSection::new();
// static CS_GUARD: Mutex<Option<IsrCriticalSectionGuard>> = Mutex::new(None);

// pub struct EspCriticalSection {}

// unsafe impl critical_section::Impl for EspCriticalSection {
//     unsafe fn acquire() {
//         let mut guard = CS_GUARD.lock().unwrap();
//         *guard = Some(CS.enter());
//     }

//     unsafe fn release(_token: ()) {
//         let mut guard = CS_GUARD.lock().unwrap();
//         *guard = None;
//     }
// }

// fn main() {
//     esp_idf_svc::sys::link_patches();
//     esp_idf_svc::log::EspLogger::initialize_default();

//     set_impl!(EspCriticalSection);

//     let peripherals = Peripherals::take().unwrap();
//     let dt = PinDriver::input(peripherals.pins.gpio2).unwrap();
//     let sck = PinDriver::output(peripherals.pins.gpio3).unwrap();

//     let delay = delay::Delay::new_default();
//     let mut load_sensor = HX711::new(sck, dt, delay);

//     load_sensor.tare(16);
//     load_sensor.set_scale(1.0);

//     loop {
//         if load_sensor.is_ready() {
//             let reading = load_sensor.read_scaled();
//             log::info!("Last Reading = {:?}", reading);
//         }

//         delay::FreeRtos::delay_ms(1000u32);
//     }
// }
