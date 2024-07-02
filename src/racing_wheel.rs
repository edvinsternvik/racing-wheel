use crate::{
    descriptor::RACING_WHEEL_DESCRIPTOR,
    hid::{GetReportInWriter, ReportWriter},
    hid_device::{HIDDeviceType, HIDReport, HIDReportOut, HIDReportRAM, ReportID},
    misc::FixedSet,
    ram_pool::{EffectParameter, RAMPool},
    reports::*,
};
use usb_device::{bus::UsbBus, UsbError};

const CUSTOM_DATA_BUFFER_SIZE: usize = 4096;
const MAX_EFFECTS: usize = 16;
const MAX_SIMULTANEOUS_EFFECTS: usize = 8;

pub struct RacingWheel {
    ram_pool: RAMPool<MAX_EFFECTS, CUSTOM_DATA_BUFFER_SIZE>,
    next_effect: Option<CreateNewEffectReport>,
    running_effects: FixedSet<u8, MAX_SIMULTANEOUS_EFFECTS>,
    device_gain: u8,
    racing_wheel_report: RacingWheelReport,
    pid_state_report: PIDStateReport,
    steering_prev: i16,
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
        }
    }

    pub fn set_steering(&mut self, steering: i16) {
        self.steering_prev = self.racing_wheel_report.steering;
        self.racing_wheel_report.steering = steering;
    }

    pub fn set_throttle(&mut self, throttle: i16) {
        self.racing_wheel_report.throttle = throttle;
    }

    pub fn set_buttons(&mut self, buttons: [bool; 8]) {
        self.racing_wheel_report.buttons = buttons;
    }

    pub fn get_force_feedback(&self) -> i32 {
        use EffectParameter::*;
        let steering_velocity = (self.racing_wheel_report.steering - self.steering_prev) as i32;

        let mut total = 0;
        for effect_block_index in self.running_effects.iter() {
            let effect = self.ram_pool.get_effect(*effect_block_index);

            if let Some(effect) = effect {
                let res = match (effect.effect_report, effect.parameter_1, effect.parameter_2) {
                    (Some(e), Some(ConstantForce(p1)), Some(Envelope(p2))) => {
                        constant_ffb(&e, &p1, &p2)
                    }
                    (Some(e), Some(Condition(p1)), Some(Condition(p2))) => {
                        damper_ffb(&e, &p1, &p2, steering_velocity)
                    }
                    _ => 0,
                };
                total += res;
            }
        }

        total
    }
}

fn constant_ffb(
    _effect: &SetEffectReport,
    constant_force: &SetConstantForceReport,
    _envelope: &SetEnvelopeReport,
) -> i32 {
    //let gain = (effect.gain as i16) * (i16::MAX / u8::MAX as i16);
    let magnitude = constant_force.magnitude;
    magnitude as i32
}

fn damper_ffb(
    _effect: &SetEffectReport,
    condition_1: &SetConditionReport,
    _condition_2: &SetConditionReport,
    velocity: i32,
) -> i32 {
    let velocity_delta = velocity as i32 - (condition_1.cp_offset - condition_1.dead_band) as i32;
    if velocity >= 0 {
        condition_1.positive_coefficient as i32 * velocity_delta
    } else {
        condition_1.negative_coefficient as i32 * velocity_delta
    }
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
                        self.running_effects.insert(report.effect_block_index);
                    }
                    EffectOperation::EffectStartSolo => {
                        self.running_effects = FixedSet::new();
                        self.running_effects.insert(report.effect_block_index);
                    }
                    EffectOperation::EffectStop => {
                        self.running_effects.remove(report.effect_block_index);
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
