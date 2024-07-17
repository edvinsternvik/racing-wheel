use cortex_m::prelude::*;
use stm32f1xx_hal::gpio::*;

// Determines whether the PWM signal is active low or active high.
pub enum PWMType {
    Normal,   // PWM is active high
    Inverted, // PWM is active low
}

pub struct Motor<PWMF> {
    forward_enable_pin: ErasedPin<Output>,
    forward_pwm: PWMF,
    pwm_type: PWMType,
}

impl<PWMF: _embedded_hal_PwmPin<Duty = u16>> Motor<PWMF> {
    pub fn new(
        forward_enable_pin: ErasedPin<Output>,
        forward_pwm: PWMF,
        pwm_type: PWMType,
    ) -> Self {
        let mut motor = Self {
            forward_enable_pin,
            forward_pwm,
            pwm_type,
        };

        motor.forward_pwm.enable();
        motor.set_speed(0, i16::MAX);

        motor
    }

    // Sets the speed of the motor using PWM. The signal will either be active low or active high
    // depending on the 'pwm_type'.
    pub fn set_speed(&mut self, speed: i16, max_speed: i16) {
        let forward_pwm_max_duty = Self::get_max_duty(&self.forward_pwm);
        let dead_band = max_speed / 100;

        let speed = i16::clamp(speed, -max_speed, max_speed);
        let speed_abs = i64::abs(speed as i64);
        let speed_normalized = (speed_abs * forward_pwm_max_duty as i64) / (max_speed as i64);
        let motor_signal = match self.pwm_type {
            PWMType::Normal => speed_normalized as u16,
            PWMType::Inverted => (forward_pwm_max_duty as i64 - speed_normalized) as u16,
        };

        if speed > dead_band {
            self.forward_enable_pin.set_high();
            self.forward_pwm.set_duty(motor_signal);
        } else if speed < -dead_band {
            // TODO
            self.forward_enable_pin.set_low();
            self.forward_pwm.set_duty(forward_pwm_max_duty as u16);
        } else {
            self.forward_enable_pin.set_low();
            self.forward_pwm.set_duty(forward_pwm_max_duty as u16);
        }
    }

    fn get_max_duty(pwm: &impl _embedded_hal_PwmPin<Duty = u16>) -> u64 {
        if pwm.get_max_duty() == 0 {
            65536
        } else {
            pwm.get_max_duty() as u64
        }
    }
}
