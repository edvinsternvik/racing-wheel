use crate::fixed::Fixed;
use cortex_m::prelude::*;
use stm32f1xx_hal::gpio::*;

const MOTOR_SIGNAL_MAX: i16 = 10_000;
const MOTOR_DEADBAND: i16 = MOTOR_SIGNAL_MAX / 1000;

pub type MotorSignal = Fixed<{ MOTOR_SIGNAL_MAX as u64 }, i16>;

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
        let speed = MotorSignal::new(i16::clamp(
            speed.value(),
            -MOTOR_SIGNAL_MAX,
            MOTOR_SIGNAL_MAX,
        ));
        let speed_abs = MotorSignal::new(speed.value().abs());

        if speed.value() > MOTOR_DEADBAND {
            let motor_signal = speed_abs.to_frac(Self::get_max_duty(&self.forward_pwm));

            self.reverse_pwm.set_duty(0);
            self.forward_pwm.set_duty(motor_signal.value() as u16);
            self.enable_pin.set_high();
        } else if speed.value() < -MOTOR_DEADBAND {
            let motor_signal = speed_abs.to_frac(Self::get_max_duty(&self.reverse_pwm));

            self.forward_pwm.set_duty(0);
            self.reverse_pwm.set_duty(motor_signal.value() as u16);
            self.enable_pin.set_high();
        } else {
            self.forward_pwm.set_duty(0);
            self.reverse_pwm.set_duty(0);
            self.enable_pin.set_low();
        }
    }

    fn get_max_duty(pwm: &impl _embedded_hal_PwmPin<Duty = u16>) -> i16 {
        if pwm.get_max_duty() == 0 {
            i16::MAX
        } else {
            pwm.get_max_duty() as i16
        }
    }
}
