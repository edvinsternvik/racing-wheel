use crate::{
    descriptor::{FORCE_LOGICAL_MAX, FORCE_LOGICAL_MIN, GAIN_MAX, RACING_WHEEL_DESCRIPTOR},
    hid::{GetReportInWriter, ReportWriter},
    hid_device::{HIDDeviceType, HIDReport, HIDReportOut, HIDReportRAM, ReportID},
    misc::FixedSet,
    ram_pool::{Effect, EffectParameter, RAMPool},
    reports::*,
};
use usb_device::{bus::UsbBus, UsbError};

const CUSTOM_DATA_BUFFER_SIZE: usize = 4096;
const MAX_EFFECTS: usize = 16;
const MAX_SIMULTANEOUS_EFFECTS: usize = 8;

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

pub struct RacingWheel {
    ram_pool: RAMPool<MAX_EFFECTS, CUSTOM_DATA_BUFFER_SIZE>,
    next_effect: Option<CreateNewEffectReport>,
    running_effects: FixedSet<RunningEffect, MAX_SIMULTANEOUS_EFFECTS>,
    device_gain: u8,
    racing_wheel_report: RacingWheelReport,
    pid_state_report: PIDStateReport,
    steering_prev: i16,
    steering_velocity: i16,
}

impl RacingWheel {
    pub fn new() -> Self {
        RacingWheel {
            ram_pool: RAMPool::new(),
            next_effect: None,
            running_effects: FixedSet::new(),
            device_gain: 0,
            racing_wheel_report: RacingWheelReport::default(),
            pid_state_report: PIDStateReport::default(),
            steering_prev: 0,
            steering_velocity: 0,
        }
    }

    // Steering angle in degrees * 10^-1
    pub fn set_steering(&mut self, steering: i16) {
        self.racing_wheel_report.steering = steering;
    }

    pub fn set_throttle(&mut self, throttle: i16) {
        self.racing_wheel_report.throttle = throttle;
    }

    pub fn set_buttons(&mut self, buttons: [bool; 8]) {
        self.racing_wheel_report.buttons = buttons;
    }

    pub fn get_force_feedback(&self) -> i16 {
        let mut total = 0;
        for running_effect in self.running_effects.iter() {
            let effect = self.ram_pool.get_effect(running_effect.index);
            let t = running_effect.time;

            if let Some(effect) = effect {
                let force = calculate_force_feedback(
                    effect,
                    t,
                    self.racing_wheel_report.steering,
                    self.steering_velocity,
                    0,
                );
                total = add_forces(total, force);
            }
        }

        apply_gain(total, self.device_gain)
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

fn calculate_force_feedback(
    effect: &Effect,
    time: u32,
    position: i16,
    velocity: i16,
    acceleration: i16,
) -> i16 {
    use EffectParameter::*;

    match (effect.effect_report, effect.parameter_1, effect.parameter_2) {
        (Some(e), Some(ConstantForce(p1)), Some(Envelope(p2))) => constant_ffb(&e, &p1, &p2, time),
        (Some(e), Some(RampForce(p1)), Some(Envelope(p2))) => ramp_ffb(&e, &p1, &p2, time),
        (Some(_), Some(CustomForce(_p)), None) => 0,
        (Some(e), Some(Periodic(p1)), Some(Envelope(p2))) => match e.effect_type {
            EffectType::Square => periodic_ffb(&e, &p1, &p2, time, square_fn),
            EffectType::Sine => periodic_ffb(&e, &p1, &p2, time, sine_fn),
            EffectType::Triangle => periodic_ffb(&e, &p1, &p2, time, triangle_fn),
            EffectType::SawtoothUp => periodic_ffb(&e, &p1, &p2, time, sawtooth_up_fn),
            EffectType::SawtoothDown => periodic_ffb(&e, &p1, &p2, time, sawtooth_down_fn),
            _ => 0,
        },
        (Some(e), Some(Condition(p1)), Some(Condition(p2))) => match e.effect_type {
            EffectType::Spring => condition_ffb(&e, &p1, &p2, position),
            EffectType::Damper => condition_ffb(&e, &p1, &p2, velocity),
            EffectType::Inertia => condition_ffb(&e, &p1, &p2, acceleration),
            EffectType::Friction => 0,
            _ => 0,
        },
        _ => 0,
    }
}

fn add_forces(force1: i16, force2: i16) -> i16 {
    i16::clamp(force1 + force2, FORCE_LOGICAL_MIN, FORCE_LOGICAL_MAX)
}

fn apply_gain(force: i16, gain: u8) -> i16 {
    ((force as i32) * (gain as i32) / (GAIN_MAX as i32)) as i16
}

fn apply_envelope(
    force: i16,
    envelope: &SetEnvelopeReport,
    time: u32,
    duration: Option<u16>,
) -> i16 {
    let calc_fade_force = |fl, ft, t| {
        let (fl, ft, m) = (fl as i64, ft as i64, FORCE_LOGICAL_MAX as i64);
        let fade = ((fl * ft + (m - fl) * t as i64) / ft) as i32;

        ((force as i32) * fade / (FORCE_LOGICAL_MAX as i32)) as i16
    };

    let duration = duration.unwrap_or(u16::MAX) as u32;

    let mut result = force;
    if time < envelope.attack_time {
        let fade_force = calc_fade_force(envelope.attack_level, envelope.attack_time, time);
        result = i16::min(result, fade_force);
    }
    if time <= duration && time + envelope.fade_time > duration {
        let fade_force = calc_fade_force(envelope.fade_level, envelope.fade_time, duration - time);
        result = i16::min(result, fade_force);
    }

    result
}

fn condition_force(metric: i16, condition: &SetConditionReport) -> i16 {
    let force = if metric < condition.cp_offset - condition.dead_band as i16 {
        let velocity_delta = metric - (condition.cp_offset - condition.dead_band as i16);
        (condition.negative_coefficient as i32 * velocity_delta as i32) / (FORCE_LOGICAL_MAX as i32)
    } else if metric > condition.cp_offset + condition.dead_band as i16 {
        let velocity_delta = metric - (condition.cp_offset + condition.dead_band as i16);
        (condition.positive_coefficient as i32 * velocity_delta as i32) / (FORCE_LOGICAL_MAX as i32)
    } else {
        0
    };

    i16::clamp(
        force as i16,
        -(condition.negative_saturation as i16),
        condition.positive_saturation as i16,
    )
}

fn constant_ffb(
    effect: &SetEffectReport,
    constant_force: &SetConstantForceReport,
    envelope: &SetEnvelopeReport,
    time: u32,
) -> i16 {
    let force = constant_force.magnitude;
    let force = apply_envelope(force, envelope, time, effect.duration);
    let force = apply_gain(force, effect.gain);
    force
}

fn ramp_ffb(
    effect: &SetEffectReport,
    ramp_force: &SetRampForceReport,
    envelope: &SetEnvelopeReport,
    time: u32,
) -> i16 {
    if let Some(duration) = effect.duration {
        let force = ramp_force.ramp_start
            + (((ramp_force.ramp_end - ramp_force.ramp_start) as i32 * time as i32)
                / duration as i32) as i16;

        let force = apply_envelope(force, envelope, time, effect.duration);
        let force = apply_gain(force, effect.gain);
        force
    } else {
        0
    }
}

fn condition_ffb(
    effect: &SetEffectReport,
    condition_1: &SetConditionReport,
    _condition_2: &SetConditionReport,
    metric: i16,
) -> i16 {
    let force = condition_force(metric, condition_1);
    let force = apply_gain(force, effect.gain);
    force
}

fn periodic_ffb(
    effect: &SetEffectReport,
    periodic: &SetPeriodicReport,
    envelope: &SetEnvelopeReport,
    time: u32,
    f: fn(u32, i16, u32) -> i16,
) -> i16 {
    let effect_time = time + ((periodic.phase as u64 * periodic.period as u64) / 36_000) as u32;
    let force = f(effect_time, periodic.magnitude as i16, periodic.period);
    let force = apply_envelope(force, envelope, time, effect.duration);
    let force = apply_gain(force, effect.gain);
    force
}

fn square_fn(t: u32, magnitude: i16, period: u32) -> i16 {
    let t = t % period;
    let period_h = period / 2;
    let r = if t >= period_h { magnitude } else { -magnitude };
    r
}

fn sine_fn(t: u32, magnitude: i16, period: u32) -> i16 {
    const LUT_SAMPLES: usize = 64;
    const SIN_LUT: [i16; LUT_SAMPLES + 1] = [
        0, 804, 1607, 2410, 3211, 4011, 4807, 5601, 6392, 7179, 7961, 8739, 9511, 10278, 11038,
        11792, 12539, 13278, 14009, 14732, 15446, 16150, 16845, 17530, 18204, 18867, 19519, 20159,
        20787, 21402, 22004, 22594, 23169, 23731, 24278, 24811, 25329, 25831, 26318, 26789, 27244,
        27683, 28105, 28510, 28897, 29268, 29621, 29955, 30272, 30571, 30851, 31113, 31356, 31580,
        31785, 31970, 32137, 32284, 32412, 32520, 32609, 32678, 32727, 32757, 32767,
    ];
    let period = period as u64;
    let mut t = (t as u64 % period) * 4;
    let mut sign = 1;
    if t >= 2 * period {
        sign = -1;
        t -= 2 * period;
    }
    if t >= period {
        t = 2 * period - t;
    }
    let index = (t as u64 * LUT_SAMPLES as u64) / period as u64;
    let force = sign * SIN_LUT[index as usize] as i32;

    ((force * magnitude as i32) / (i16::MAX as i32)) as i16
}

fn triangle_fn(t: u32, magnitude: i16, period: u32) -> i16 {
    let period = period as i64;
    let t = (t as i64 % period) * 2;
    let t = if t < period { t } else { 2 * period - t };
    ((2 * t * magnitude as i64) / period as i64) as i16 - magnitude
}

fn sawtooth_up_fn(t: u32, magnitude: i16, period: u32) -> i16 {
    let period = period as i64;
    let t = t as i64 % period;
    ((2 * t * magnitude as i64) / period as i64) as i16 - magnitude
}

fn sawtooth_down_fn(t: u32, magnitude: i16, period: u32) -> i16 {
    -sawtooth_up_fn(t, magnitude, period)
}

impl HIDDeviceType for RacingWheel {
    fn descriptor() -> &'static [u8] {
        RACING_WHEEL_DESCRIPTOR
    }

    fn get_report_request<B: UsbBus>(
        &mut self,
        report_id: ReportID,
        writer: GetReportInWriter<B>,
    ) -> Result<(), UsbError> {
        match report_id {
            PIDBlockLoadReport::ID => {
                if let Some(_) = self.next_effect {
                    self.next_effect = None;

                    if let Some(index) = self.ram_pool.new_effect() {
                        writer.accept(PIDBlockLoadReport {
                            effect_block_index: index as u8,
                            block_load_status: BlockLoadStatus::Success,
                            ram_pool_available: self.ram_pool.available() as u16,
                        })?;
                    } else {
                        writer.accept(PIDBlockLoadReport {
                            effect_block_index: 0,
                            block_load_status: BlockLoadStatus::Full,
                            ram_pool_available: self.ram_pool.available() as u16,
                        })?;
                    }
                } else {
                    writer.accept(PIDBlockLoadReport {
                        effect_block_index: 0,
                        block_load_status: BlockLoadStatus::Error,
                        ram_pool_available: self.ram_pool.available() as u16,
                    })?;
                }
                Ok(())
            }
            PIDPoolReport::ID => writer.accept(PIDPoolReport {
                ram_pool_size: self.ram_pool.pool_size() as u16,
                simultaneous_effects_max: MAX_SIMULTANEOUS_EFFECTS as u8,
                param_block_size_set_effect: SetEffectReport::RAM_SIZE as u8,
                param_block_size_set_envelope: SetEnvelopeReport::RAM_SIZE as u8,
                param_block_size_set_condition: SetConditionReport::RAM_SIZE as u8,
                param_block_size_set_periodic: SetPeriodicReport::RAM_SIZE as u8,
                param_block_size_set_constant_force: SetConstantForceReport::RAM_SIZE as u8,
                param_block_size_set_ramp_force: SetRampForceReport::RAM_SIZE as u8,
                param_block_size_set_custom_force: SetCustomForceReport::RAM_SIZE as u8,
                device_managed_pool: true,
                shared_parameter_blocks: false,
                isochronous_enable: true,
            }),
            _ => Ok(()),
        }
    }

    fn report_request_out(&mut self, report_id: ReportID, data: &[u8]) -> Result<Option<bool>, ()> {
        match report_id {
            SetEffectReport::ID => {
                let report = SetEffectReport::into_report(data).ok_or(())?;
                let effect = self
                    .ram_pool
                    .get_effect_mut(report.effect_block_index)
                    .ok_or(())?;
                effect.effect_report = Some(report);

                Ok(Some(true))
            }
            SetEnvelopeReport::ID => {
                let report = SetEnvelopeReport::into_report(data).ok_or(())?;
                let effect = self
                    .ram_pool
                    .get_effect_mut(report.effect_block_index)
                    .ok_or(())?;
                effect.parameter_2 = Some(EffectParameter::Envelope(report));

                Ok(Some(true))
            }
            SetConditionReport::ID => {
                let report = SetConditionReport::into_report(data).ok_or(())?;
                let effect = self
                    .ram_pool
                    .get_effect_mut(report.effect_block_index)
                    .ok_or(())?;
                if report.parameter_block_offset == 0 {
                    effect.parameter_1 = Some(EffectParameter::Condition(report));
                } else if report.parameter_block_offset == 1 {
                    effect.parameter_2 = Some(EffectParameter::Condition(report));
                }
                Ok(Some(true))
            }
            SetPeriodicReport::ID => {
                let report = SetPeriodicReport::into_report(data).ok_or(())?;
                let effect = self
                    .ram_pool
                    .get_effect_mut(report.effect_block_index)
                    .ok_or(())?;
                effect.parameter_1 = Some(EffectParameter::Periodic(report));

                Ok(Some(true))
            }
            SetConstantForceReport::ID => {
                let report = SetConstantForceReport::into_report(data).ok_or(())?;
                let effect = self
                    .ram_pool
                    .get_effect_mut(report.effect_block_index)
                    .ok_or(())?;
                effect.parameter_1 = Some(EffectParameter::ConstantForce(report));

                Ok(Some(true))
            }
            SetRampForceReport::ID => {
                let report = SetRampForceReport::into_report(data).ok_or(())?;
                let effect = self
                    .ram_pool
                    .get_effect_mut(report.effect_block_index)
                    .ok_or(())?;
                effect.parameter_1 = Some(EffectParameter::RampForce(report));

                Ok(Some(true))
            }
            CustomForceDataReport::ID => {
                let _ = CustomForceDataReport::into_report(data).ok_or(())?;
                Ok(Some(true))
            }
            EffectOperationReport::ID => {
                let report = EffectOperationReport::into_report(data).ok_or(())?;
                match report.effect_operation {
                    EffectOperation::EffectStart => {
                        self.running_effects
                            .insert(RunningEffect::new(report.effect_block_index));
                    }
                    EffectOperation::EffectStartSolo => {
                        self.running_effects = FixedSet::new();
                        self.running_effects
                            .insert(RunningEffect::new(report.effect_block_index));
                    }
                    EffectOperation::EffectStop => {
                        self.running_effects
                            .remove(RunningEffect::new(report.effect_block_index));
                    }
                }

                Ok(Some(true))
            }
            PIDBlockFreeReport::ID => {
                let report = PIDBlockFreeReport::into_report(data).ok_or(())?;
                self.ram_pool.free_effect(report.effect_block_index)?;
                Err(())
            }
            PIDDeviceControl::ID => {
                let report = PIDDeviceControl::into_report(data).ok_or(())?;
                match report.device_control {
                    DeviceControl::EnableActuators => {
                        self.pid_state_report.actuators_enabled = true
                    }
                    DeviceControl::DisableActuators => {
                        self.pid_state_report.actuators_enabled = false
                    }
                    DeviceControl::StopAllEffects => self.running_effects = FixedSet::new(),
                    DeviceControl::DeviceReset => *self = Self::new(),
                    DeviceControl::DevicePause => self.pid_state_report.device_paused = true,
                    DeviceControl::DeviceContinue => self.pid_state_report.device_paused = false,
                }

                Ok(Some(true))
            }
            DeviceGainReport::ID => {
                let report = DeviceGainReport::into_report(data).ok_or(())?;
                self.device_gain = report.device_gain;
                Ok(Some(true))
            }
            SetCustomForceReport::ID => {
                let report = SetCustomForceReport::into_report(data).ok_or(())?;
                let effect = self
                    .ram_pool
                    .get_effect_mut(report.effect_block_index)
                    .ok_or(())?;
                effect.parameter_1 = Some(EffectParameter::CustomForce(report));

                Ok(Some(true))
            }
            PIDPoolMoveReport::ID => {
                let _ = PIDPoolMoveReport::into_report(data).ok_or(())?;
                Ok(Some(true))
            }
            CreateNewEffectReport::ID => {
                let report = CreateNewEffectReport::into_report(data).ok_or(())?;
                self.next_effect = Some(report);
                Ok(Some(true))
            }
            _ => Ok(None),
        }
    }

    fn send_input_reports<B: UsbBus>(&mut self, writer: ReportWriter<B>) -> Result<(), UsbError> {
        writer.write_report(self.racing_wheel_report.clone())?;
        writer.write_report(self.pid_state_report.clone())?;

        Ok(())
    }
}
