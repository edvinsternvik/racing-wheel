mod descriptor;
mod hid_reports;
mod racing_wheel_hid;
mod ram_pool;

use crate::{config::Config, misc::FixedSet};
use force_feedback::{
    // effect::{create_spring_effect, Effect, EffectParameter},
    effect::create_spring_effect,
    ffb::calculate_force_feedback,
    reports::*,
};
use micromath::F32Ext;
use ram_pool::RAMPool;

const CUSTOM_DATA_BUFFER_SIZE: usize = 4096;
const MAX_EFFECTS: usize = 16;
const MAX_SIMULTANEOUS_EFFECTS: usize = 8;

pub struct RacingWheel {
    ram_pool: RAMPool<MAX_EFFECTS, CUSTOM_DATA_BUFFER_SIZE>,
    next_effect: Option<CreateNewEffect>,
    running_effects: FixedSet<RunningEffect, MAX_SIMULTANEOUS_EFFECTS>,
    device_gain: f32,
    racing_wheel_report: RacingWheelState,
    pid_state_report: PIDState,
    steering_prev: f32,
    steering_velocity: f32,
    config: Config,
    write_config_event: bool,
    reboot_device_event: bool,
    reset_steering_event: bool,
}

impl RacingWheel {
    pub fn new(config: Config) -> Self {
        RacingWheel {
            ram_pool: RAMPool::new(),
            next_effect: None,
            running_effects: FixedSet::new(),
            device_gain: 0.0,
            racing_wheel_report: RacingWheelState::default(),
            pid_state_report: PIDState::default(),
            steering_prev: 0.0,
            steering_velocity: 0.0,
            config,
            write_config_event: false,
            reboot_device_event: false,
            reset_steering_event: false,
        }
    }

    // Steering angle (degrees)
    pub fn set_steering(&mut self, steering: f32) {
        self.racing_wheel_report.steering = steering * 2.0 / (self.config.max_rotation as f32);
    }

    pub fn set_buttons(&mut self, buttons: [bool; 8]) {
        self.racing_wheel_report.buttons = buttons;
    }

    pub fn get_config(&self) -> Config {
        self.config
    }

    pub fn write_config_event(&mut self) -> bool {
        let write_config = self.write_config_event;
        self.write_config_event = false;
        write_config
    }

    pub fn reboot_device_event(&mut self) -> bool {
        let reboot_device = self.reboot_device_event;
        self.reboot_device_event = false;
        reboot_device
    }

    pub fn reset_steering_event(&mut self) -> bool {
        if self.reset_steering_event {
            self.reset_steering_event = false;

            self.racing_wheel_report.steering = 0.0;
            self.steering_prev = 0.0;
            self.steering_velocity = 0.0;

            return true;
        }

        false
    }

    pub fn get_force_feedback(&self) -> f32 {
        let mut total: f32 = 0.0;

        // Apply PID effects
        for running_effect in self.running_effects.iter() {
            let effect = self.ram_pool.get_effect(running_effect.index);
            let t = running_effect.time;

            if let Some(effect) = effect {
                let force = calculate_force_feedback(
                    effect,
                    t,
                    self.racing_wheel_report.steering,
                    self.steering_velocity,
                    0.0,
                );
                total = total + force;
            }
        }

        // Apply spring effect
        total = total
            + calculate_force_feedback(
                &create_spring_effect(
                    self.config.spring_gain,
                    None,
                    0.0,
                    self.config.spring_coefficient,
                    self.config.spring_coefficient,
                    self.config.spring_saturation,
                    self.config.spring_saturation,
                    self.config.spring_deadband,
                ),
                0,
                self.racing_wheel_report.steering,
                0.0,
                0.0,
            );

        let ffb = total * self.device_gain * self.config.gain * self.config.motor_max;
        f32::signum(ffb) * f32::powf(f32::abs(ffb), self.config.expo)
    }

    pub fn advance(&mut self, delta_time_ms: u32) {
        self.steering_velocity = (self.racing_wheel_report.steering - self.steering_prev)
            * (delta_time_ms as f32 / 1000.0);
        self.steering_prev = self.racing_wheel_report.steering;

        let mut still_running = FixedSet::new();
        for running_effect in self.running_effects.iter_mut() {
            running_effect.time += delta_time_ms;

            let mut keep = true;
            if let Some(effect) = self.ram_pool.get_effect(running_effect.index) {
                if let Some(duration) = effect.effect_report.and_then(|e| e.duration) {
                    keep = keep && duration as u32 > running_effect.time;
                }
                if running_effect.time > 10_000 && !effect.is_complete() {
                    keep = false;
                }
            }

            if keep {
                still_running.insert(*running_effect);
            }
        }

        self.running_effects = still_running;
    }
}

#[derive(Copy, Clone, Eq, Default)]
struct RunningEffect {
    index: u8,
    time: u32,
}

impl PartialEq for RunningEffect {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}

impl RunningEffect {
    fn new(index: u8) -> Self {
        Self { index, time: 0 }
    }
}
