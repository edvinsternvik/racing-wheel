use super::{
    descriptor::RACING_WHEEL_DESCRIPTOR, RacingWheel, RunningEffect, MAX_SIMULTANEOUS_EFFECTS,
};
use crate::{
    misc::FixedSet,
    usb::{
        hid::{GetReportInWriter, ReportWriter},
        hid_device::{HIDDeviceType, HIDReport, HIDReportOut, HIDReportRAM, ReportID},
    },
};
use force_feedback::{effect::EffectParameter, reports::*};
use usb_device::{bus::UsbBus, UsbError};

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
            SetConfigReport::ID => {
                self.config = SetConfigReport::into_report(data).ok_or(())?;
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
