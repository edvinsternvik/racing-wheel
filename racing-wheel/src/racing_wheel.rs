mod descriptor;
mod hid_reports;
mod racing_wheel_hid;
mod ram_pool;

use crate::misc::FixedSet;
use fixed_num::Frac16;
use force_feedback::{
    effect::{create_spring_effect, Effect, EffectParameter},
    ffb::calculate_force_feedback,
    reports::*,
};
use ram_pool::RAMPool;

const CUSTOM_DATA_BUFFER_SIZE: usize = 4096;
const MAX_EFFECTS: usize = 16;
const MAX_SIMULTANEOUS_EFFECTS: usize = 8;
const DEGREES_OF_ROTATION: i16 = 900;

pub struct RacingWheel {
    ram_pool: RAMPool<MAX_EFFECTS, CUSTOM_DATA_BUFFER_SIZE>,
    next_effect: Option<CreateNewEffect>,
    running_effects: FixedSet<RunningEffect, MAX_SIMULTANEOUS_EFFECTS>,
    device_gain: FixedFFB,
    racing_wheel_report: RacingWheelState,
    pid_state_report: PIDState,
    steering_prev: FixedSteering,
    steering_velocity: FixedSteering,
    config: SetConfig,
}

impl RacingWheel {
    pub fn new() -> Self {
        RacingWheel {
            ram_pool: RAMPool::new(),
            next_effect: None,
            running_effects: FixedSet::new(),
            device_gain: 0.into(),
            racing_wheel_report: RacingWheelState::default(),
            pid_state_report: PIDState::default(),
            steering_prev: 0.into(),
            steering_velocity: 0.into(),
            config: SetConfig {
                gain: Frac16::new(1, 4).convert(),
            },
        }
    }

    // Steering angle in unit of full revolutions
    pub fn set_steering(&mut self, steering: FixedSteering) {
        self.racing_wheel_report.steering = steering * Frac16::new(2 * 360, DEGREES_OF_ROTATION);
    }

    pub fn set_buttons(&mut self, buttons: [bool; 8]) {
        self.racing_wheel_report.buttons = buttons;
    }

    pub fn get_force_feedback(&self) -> FixedFFB {
        let mut total: FixedFFB = 0.into();

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
                    0.into(),
                );
                total = total + force;
            }
        }

        // Apply spring effect
        total = total + calculate_force_feedback(
            &create_spring_effect(
                Frac16::new(4, 1).convert(),
                None,
                0.into(),
                FixedFFB::one(),
                FixedFFB::one(),
                Frac16::new(1, 4).convert(),
                Frac16::new(1, 4).convert(),
                0.into(),
            ),
            0,
            self.racing_wheel_report.steering,
            self.steering_velocity,
            0.into(),
        );

        total * self.device_gain * self.config.gain
    }

    pub fn advance(&mut self, delta_time_ms: u32) {
        self.steering_velocity =
            (self.racing_wheel_report.steering - self.steering_prev) * delta_time_ms as i16;
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
