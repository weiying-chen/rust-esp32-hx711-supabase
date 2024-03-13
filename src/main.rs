use anyhow::Result;
use std::collections::VecDeque;

use esp_idf_svc::hal::{
    delay::{Ets, FreeRtos},
    gpio::{Input, InputPin, Output, OutputPin, Pin, PinDriver},
    peripheral::Peripheral,
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
const LOADCELL_SCALING: f32 = 0.0027;

pub type LoadSensor<'a, SckPin, DtPin> =
    HX711<PinDriver<'a, SckPin, Output>, PinDriver<'a, DtPin, Input>, Ets>;

/// Loadcell struct
pub struct Scale<'a, SckPin, DtPin>
where
    DtPin: Peripheral<P = DtPin> + Pin + InputPin,
    SckPin: Peripheral<P = SckPin> + Pin + OutputPin,
{
    load_sensor: LoadSensor<'a, SckPin, DtPin>,
}

impl<'a, SckPin, DtPin> Scale<'a, SckPin, DtPin>
where
    DtPin: Peripheral<P = DtPin> + Pin + InputPin,
    SckPin: Peripheral<P = SckPin> + Pin + OutputPin,
{
    pub fn new(clock_pin: SckPin, data_pin: DtPin) -> Result<Self> {
        let dt = PinDriver::input(data_pin)?;
        let sck = PinDriver::output(clock_pin)?;
        let mut load_sensor = HX711::new(sck, dt, Ets);

        load_sensor.set_scale(LOADCELL_SCALING);

        Ok(Scale { load_sensor })
    }

    pub fn is_ready(&self) -> bool {
        self.load_sensor.is_ready()
    }

    pub fn set_scale(&mut self, scale: f32) {
        self.load_sensor.set_scale(scale);
    }

    pub fn tare(&mut self, times: usize) {
        self.load_sensor.tare(times);
    }

    pub fn read_scaled(&mut self) -> Result<f32, NotReadyError> {
        self.load_sensor.read_scaled()
    }

    pub fn read(&mut self) -> Result<i32, NotReadyError> {
        self.load_sensor.read()
    }

    pub fn wait_stable(&mut self) {
        let mut readings: VecDeque<f32> = VecDeque::with_capacity(LOADCELL_STABLE_READINGS);

        loop {
            while !self.load_sensor.is_ready() {
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
}

fn main() {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();
    let dt = peripherals.pins.gpio2;
    let sck = peripherals.pins.gpio3;

    let mut scale = Scale::new(sck, dt).unwrap();
    scale.wait_stable();
    scale.tare(32);

    loop {
        if scale.is_ready() {
            let reading = scale.read_scaled().unwrap();
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
