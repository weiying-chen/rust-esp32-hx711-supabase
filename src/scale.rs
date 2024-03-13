use std::collections::VecDeque;

use anyhow::Result;
use esp_idf_svc::hal::{
    delay::{Ets, FreeRtos},
    gpio::{Input, InputPin, Output, OutputPin, Pin, PinDriver},
    peripheral::Peripheral,
};
use loadcell::{
    hx711::{NotReadyError, HX711},
    LoadCell,
};

const LOADCELL_STABLE_READINGS: usize = 10; // Number of readings to consider for stability
const LOADCELL_LOOP_DELAY_US: u32 = 10000; // Delay between readings in microseconds
                                           // const LOADCELL_READY_DELAY_US: u32 = 1000;
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
    pub fn new(clock_pin: SckPin, data_pin: DtPin, scaling: f32) -> Result<Self> {
        let dt = PinDriver::input(data_pin)?;
        let sck = PinDriver::output(clock_pin)?;
        let mut load_sensor = HX711::new(sck, dt, Ets);

        load_sensor.set_scale(scaling);

        Ok(Scale { load_sensor })
    }

    pub fn is_ready(&self) -> bool {
        self.load_sensor.is_ready()
    }

    // fn set_scale(&mut self, scale: f32) {
    //     self.load_sensor.set_scale(scale);
    // }

    pub fn tare(&mut self, times: usize) {
        self.load_sensor.tare(times);
    }

    // pub fn read_scaled(&mut self) -> Result<f32, NotReadyError> {
    //     self.load_sensor.read_scaled()
    // }

    // fn read(&mut self) -> Result<i32, NotReadyError> {
    //     self.load_sensor.read()
    // }

    pub fn read_rounded(&mut self) -> Result<i32, NotReadyError> {
        let reading = self.load_sensor.read_scaled()?;

        let rounded_reading = if reading.round() == -0f32 {
            0
        } else {
            reading.round() as i32
        };

        Ok(rounded_reading)
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
