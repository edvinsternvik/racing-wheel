use cortex_m::prelude::*;
use stm32f1xx_hal::gpio::*;

pub struct Motor<PWMF, PWMR> {
    enable_pin: ErasedPin<Output>,
    forward_pwm: PWMF,
    reverse_pwm: PWMR,
}

impl<PWMF: _embedded_hal_PwmPin<Duty = u16>, PWMR: _embedded_hal_PwmPin<Duty = u16>>
    Motor<PWMF, PWMR>
{
    pub fn new(enable_pin: ErasedPin<Output>, forward_pwm: PWMF, reverse_pwm: PWMR) -> Self {
        let mut motor = Self {
            enable_pin,
            forward_pwm,
            reverse_pwm,
        };

        motor.forward_pwm.enable();
        motor.reverse_pwm.enable();
        motor.set_speed(0.0, (0.0, 0.0), 1.0);

        motor
    }

    pub fn set_speed(&mut self, speed: f32, speed_range: (f32, f32), deadband: f32) {
        let max_speed = f32::clamp(speed_range.1, 0.0, 1.0);
        let min_speed = f32::clamp(speed_range.0, 0.0, max_speed);
        let speed = f32::clamp(speed, -1.0, 1.0);
        let speed_abs = if speed >= 0.0 { speed } else { -speed };
        let speed_signal = speed_abs * (max_speed - min_speed) + min_speed;

        if speed > deadband {
            let motor_signal = speed_signal * Self::get_max_duty(&self.forward_pwm);

            self.reverse_pwm.set_duty(0);
            self.forward_pwm.set_duty(motor_signal as u16);
            self.enable_pin.set_high();
        } else if speed < -deadband {
            let motor_signal = speed_signal * Self::get_max_duty(&self.reverse_pwm);

            self.forward_pwm.set_duty(0);
            self.reverse_pwm.set_duty(motor_signal as u16);
            self.enable_pin.set_high();
        } else {
            self.forward_pwm.set_duty(0);
            self.reverse_pwm.set_duty(0);
            self.enable_pin.set_low();
        }
    }

    fn get_max_duty(pwm: &impl _embedded_hal_PwmPin<Duty = u16>) -> f32 {
        if pwm.get_max_duty() == 0 {
            i16::MAX as f32
        } else {
            pwm.get_max_duty() as f32
        }
    }
}
