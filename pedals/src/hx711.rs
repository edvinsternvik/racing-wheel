use cortex_m::asm::delay;
use stm32f1xx_hal::gpio::{ErasedPin, Input, Output};

pub struct HX711 {
    sysclk_hz: u32,
    d_out: ErasedPin<Input>,
    pd_sck: ErasedPin<Output>,
}

impl HX711 {
    pub fn new(d_out: ErasedPin<Input>, mut pd_sck: ErasedPin<Output>, sysclk_hz: u32) -> Self {
        pd_sck.set_low();
        Self {
            sysclk_hz,
            d_out,
            pd_sck,
        }
    }

    pub fn data_available(&self) -> bool {
        self.d_out.is_low()
    }

    pub fn read_data(&mut self) -> f32 {
        let data_bits = 24;
        let one_microsecond = self.sysclk_hz / 1_000_000;
        delay(one_microsecond);

        let mut bits = 0;
        for _ in 0..data_bits {
            self.pd_sck.set_high();
            delay(one_microsecond);
            self.pd_sck.set_low();

            bits = (bits << 1) + self.d_out.is_high() as i32;
        }
        self.pd_sck.set_high();
        delay(one_microsecond);
        self.pd_sck.set_low();

        let sign = bits & (1 << (data_bits - 1));

        let data = ((bits ^ sign) - sign) as f32;
        let data_max = (1 << data_bits) as f32;
        let brake = f32::clamp(data / data_max, 0.0, 1.0);

        brake
    }
}
