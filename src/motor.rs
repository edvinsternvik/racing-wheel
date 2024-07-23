use crate::misc::{DSignal, Signal};
use cortex_m::prelude::*;
use stm32f1xx_hal::gpio::*;

const MOTOR_SIGNAL_MAX: i32 = 10_000;
const MOTOR_SIGNAL_MIN: i32 = -10_000;
const MOTOR_DEADBAND_MAX: i32 = MOTOR_SIGNAL_MAX / 100;
const MOTOR_DEADBAND_MIN: i32 = MOTOR_SIGNAL_MIN / 100;

pub type MotorSignal = Signal<MOTOR_SIGNAL_MIN, MOTOR_SIGNAL_MAX>;

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
        motor.set_speed(0.into());

        motor
    }

    // Sets the speed of the motor using PWM. The signal will either be active low or active high
    // depending on the 'pwm_type'.
    pub fn set_speed(&mut self, speed: MotorSignal) {
        let speed_abs = DSignal::new(speed.value().abs(), 0, MOTOR_SIGNAL_MAX);

        if speed.value() > MOTOR_DEADBAND_MAX {
            let motor_signal =
                DSignal::from(speed_abs).convert(0, Self::get_max_duty(&self.forward_pwm));

            self.reverse_pwm.set_duty(0);
            self.forward_pwm.set_duty(motor_signal.value() as u16);
            self.enable_pin.set_high();
        } else if speed.value() < MOTOR_DEADBAND_MIN {
            let motor_signal =
                DSignal::from(speed_abs).convert(0, Self::get_max_duty(&self.reverse_pwm));

            self.forward_pwm.set_duty(0);
            self.reverse_pwm.set_duty(motor_signal.value() as u16);
            self.enable_pin.set_high();
        } else {
            self.forward_pwm.set_duty(0);
            self.reverse_pwm.set_duty(0);
            self.enable_pin.set_low();
        }
    }

    fn get_max_duty(pwm: &impl _embedded_hal_PwmPin<Duty = u16>) -> i32 {
        if pwm.get_max_duty() == 0 {
            65536
        } else {
            pwm.get_max_duty() as i32
        }
    }
}
