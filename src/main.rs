use std::collections::VecDeque;

use esp_idf_svc::hal::{
    delay::{Delay, Ets, FreeRtos},
    gpio::{Gpio2, Gpio3, Input, Output, PinDriver},
    peripherals::Peripherals,
};

use loadcell::{
    hx711::{NotReadyError, HX711},
    LoadCell,
};

mod critical_section;

const LOADCELL_STABLE_READINGS: usize = 10; // Number of readings to consider for stability
const LOADCELL_LOOP_DELAY_US: u32 = 10000; // Delay between readings in microseconds
                                           // const LOADCELL_READY_DELAY_US: u32 = 1000;
pub type LoadCellAmp<'a> = HX711<PinDriver<'a, Gpio2, Output>, PinDriver<'a, Gpio3, Input>, Delay>;

pub struct LoadSensor<'a> {
    load_sensor: LoadCellAmp<'a>,
}

impl<'a> LoadSensor<'a> {
    pub fn new(gpio_sck: Gpio3, gpio_dt: Gpio2) -> Self {
        let dt = PinDriver::input(gpio_sck).unwrap();
        let sck = PinDriver::output(gpio_dt).unwrap();
        let delay = Delay::new_default();
        let load_sensor = HX711::new(sck, dt, delay);

        LoadSensor { load_sensor }
    }

    // Delegate to HX711::is_ready
    pub fn is_ready(&self) -> bool {
        self.load_sensor.is_ready()
    }

    // Delegate to HX711::set_scale
    pub fn set_scale(&mut self, scale: f32) {
        self.load_sensor.set_scale(scale);
    }

    // Delegate to HX711::tare
    pub fn tare(&mut self, times: usize) {
        self.load_sensor.tare(times);
    }

    // Delegate to HX711::read_scaled
    pub fn read_scaled(&mut self) -> Result<f32, NotReadyError> {
        self.load_sensor.read_scaled()
    }

    // Optionally, delegate to HX711::read for raw readings useful in calibration
    pub fn read(&mut self) -> Result<i32, NotReadyError> {
        self.load_sensor.read()
    }

    pub fn wait_stable(&mut self) {
        let mut readings: VecDeque<f32> = VecDeque::with_capacity(LOADCELL_STABLE_READINGS);

        loop {
            while !self.load_sensor.is_ready() {
                // Use your specific delay function here, adjusted for your RTOS/environment
                FreeRtos::delay_ms(10);
            }

            let reading = self
                .load_sensor
                .read_scaled()
                .expect("Failed to read scale");

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

            Ets::delay_us(LOADCELL_LOOP_DELAY_US);
        }
    }

    // Add other methods as needed, directly utilizing `load_sensor`'s methods.
}

fn main() {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();
    let dt = peripherals.pins.gpio2;
    let sck = peripherals.pins.gpio3;

    let mut load_sensor = LoadSensor::new(sck, dt);
    // let dt = PinDriver::input(peripherals.pins.gpio2).unwrap();
    // let sck = PinDriver::output(peripherals.pins.gpio3).unwrap();
    // let delay = Delay::new_default();

    // let mut load_sensor = HX711::new(sck, dt, delay);

    // load_sensor.tare(16);
    // load_sensor.set_scale(1.0);
    // load_sensor.set_scale(0.0043);
    load_sensor.set_scale(0.0027);
    load_sensor.wait_stable();
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
