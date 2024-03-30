use anyhow::Result;
use esp_idf_svc::hal::{
    delay::Ets,
    gpio::{Input, InputPin, Output, OutputPin, Pin, PinDriver},
    peripheral::Peripheral,
};
use loadcell::{
    hx711::{NotReadyError, HX711},
    LoadCell,
};

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

    pub fn tare(&mut self, times: usize) {
        self.load_sensor.tare(times);
    }

    pub fn read_rounded(&mut self) -> Result<i32, NotReadyError> {
        let reading = self.load_sensor.read_scaled()?;

        let rounded_reading = if reading.round() == -0f32 {
            0
        } else {
            reading.round() as i32
        };

        Ok(rounded_reading)
    }
}
