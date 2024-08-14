use super::{
    descriptor::RACING_WHEEL_DESCRIPTOR, hid_reports::Report, RacingWheel, RunningEffect,
    MAX_SIMULTANEOUS_EFFECTS,
};
use crate::misc::FixedSet;
use force_feedback::{effect::EffectParameter, reports::*};
use usb_device::{bus::UsbBus, UsbError};
use usb_hid_device::{
    hid::{GetReportInWriter, ReportWriter},
    hid_device::{HIDDeviceType, HIDReport, HIDReportOut, HIDReportRAM, ReportID},
};

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
            Report::<PIDBlockLoad>::ID => {
                if let Some(_) = self.next_effect {
                    self.next_effect = None;

                    if let Some(index) = self.ram_pool.new_effect() {
                        writer.accept(Report(PIDBlockLoad {
                            effect_block_index: index as u8,
                            block_load_status: BlockLoadStatus::Success,
                            ram_pool_available: self.ram_pool.available() as u16,
                        }))?;
                    } else {
                        writer.accept(Report(PIDBlockLoad {
                            effect_block_index: 0,
                            block_load_status: BlockLoadStatus::Full,
                            ram_pool_available: self.ram_pool.available() as u16,
                        }))?;
                    }
                } else {
                    writer.accept(Report(PIDBlockLoad {
                        effect_block_index: 0,
                        block_load_status: BlockLoadStatus::Error,
                        ram_pool_available: self.ram_pool.available() as u16,
                    }))?;
                }
                Ok(())
            }
            Report::<PIDPool>::ID => writer.accept(Report(PIDPool {
                ram_pool_size: self.ram_pool.pool_size() as u16,
                simultaneous_effects_max: MAX_SIMULTANEOUS_EFFECTS as u8,
                param_block_size_set_effect: Report::<SetEffect>::RAM_SIZE as u8,
                param_block_size_set_envelope: Report::<SetEnvelope>::RAM_SIZE as u8,
                param_block_size_set_condition: Report::<SetCondition>::RAM_SIZE as u8,
                param_block_size_set_periodic: Report::<SetPeriodic>::RAM_SIZE as u8,
                param_block_size_set_constant_force: Report::<SetConstantForce>::RAM_SIZE as u8,
                param_block_size_set_ramp_force: Report::<SetRampForce>::RAM_SIZE as u8,
                param_block_size_set_custom_force: Report::<SetCustomForce>::RAM_SIZE as u8,
                device_managed_pool: true,
                shared_parameter_blocks: false,
                isochronous_enable: true,
            })),
            Report::<Config>::ID => writer.accept(Report(self.get_config())),
            _ => Ok(()),
        }
    }

    fn report_request_out(&mut self, report_id: ReportID, data: &[u8]) -> Result<Option<bool>, ()> {
        match report_id {
            Report::<SetEffect>::ID => {
                let report = Report::<SetEffect>::into_report(data).ok_or(())?;
                let effect = self
                    .ram_pool
                    .get_effect_mut(report.effect_block_index)
                    .ok_or(())?;
                effect.effect_report = Some(*report);

                Ok(Some(true))
            }
            Report::<SetEnvelope>::ID => {
                let report = Report::<SetEnvelope>::into_report(data).ok_or(())?;
                let effect = self
                    .ram_pool
                    .get_effect_mut(report.effect_block_index)
                    .ok_or(())?;
                effect.parameter_2 = Some(EffectParameter::Envelope(*report));

                Ok(Some(true))
            }
            Report::<SetCondition>::ID => {
                let report = Report::<SetCondition>::into_report(data).ok_or(())?;
                let effect = self
                    .ram_pool
                    .get_effect_mut(report.effect_block_index)
                    .ok_or(())?;
                if report.parameter_block_offset == 0 {
                    effect.parameter_1 = Some(EffectParameter::Condition(*report));
                } else if report.parameter_block_offset == 1 {
                    effect.parameter_2 = Some(EffectParameter::Condition(*report));
                }
                Ok(Some(true))
            }
            Report::<SetPeriodic>::ID => {
                let report = Report::<SetPeriodic>::into_report(data).ok_or(())?;
                let effect = self
                    .ram_pool
                    .get_effect_mut(report.effect_block_index)
                    .ok_or(())?;
                effect.parameter_1 = Some(EffectParameter::Periodic(*report));

                Ok(Some(true))
            }
            Report::<SetConstantForce>::ID => {
                let report = Report::<SetConstantForce>::into_report(data).ok_or(())?;
                let effect = self
                    .ram_pool
                    .get_effect_mut(report.effect_block_index)
                    .ok_or(())?;
                effect.parameter_1 = Some(EffectParameter::ConstantForce(*report));

                Ok(Some(true))
            }
            Report::<SetRampForce>::ID => {
                let report = Report::<SetRampForce>::into_report(data).ok_or(())?;
                let effect = self
                    .ram_pool
                    .get_effect_mut(report.effect_block_index)
                    .ok_or(())?;
                effect.parameter_1 = Some(EffectParameter::RampForce(*report));

                Ok(Some(true))
            }
            Report::<CustomForceData>::ID => {
                let _ = Report::<CustomForceData>::into_report(data).ok_or(())?;
                Ok(Some(true))
            }
            Report::<SetEffectOperation>::ID => {
                let report = Report::<SetEffectOperation>::into_report(data).ok_or(())?;
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
            Report::<PIDBlockFree>::ID => {
                let report = Report::<PIDBlockFree>::into_report(data).ok_or(())?;
                self.ram_pool.free_effect(report.effect_block_index)?;
                Err(())
            }
            Report::<PIDDeviceControl>::ID => {
                let report = Report::<PIDDeviceControl>::into_report(data).ok_or(())?;
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
            Report::<DeviceGain>::ID => {
                let report = Report::<DeviceGain>::into_report(data).ok_or(())?;
                self.device_gain = report.device_gain;
                Ok(Some(true))
            }
            Report::<SetCustomForce>::ID => {
                let report = Report::<SetCustomForce>::into_report(data).ok_or(())?;
                let effect = self
                    .ram_pool
                    .get_effect_mut(report.effect_block_index)
                    .ok_or(())?;
                effect.parameter_1 = Some(EffectParameter::CustomForce(*report));

                Ok(Some(true))
            }
            Report::<PIDPoolMove>::ID => {
                let _ = Report::<PIDPoolMove>::into_report(data).ok_or(())?;
                Ok(Some(true))
            }
            Report::<CreateNewEffect>::ID => {
                let report = Report::<CreateNewEffect>::into_report(data).ok_or(())?;
                self.next_effect = Some(*report);
                Ok(Some(true))
            }
            Report::<Config>::ID => {
                self.config = *Report::<Config>::into_report(data).ok_or(())?;
                Ok(Some(true))
            }
            _ => Ok(None),
        }
    }

    fn send_input_reports<B: UsbBus>(&mut self, writer: ReportWriter<B>) -> Result<(), UsbError> {
        writer.write_report(Report(self.racing_wheel_report.clone()))?;
        writer.write_report(Report(self.pid_state_report.clone()))?;

        Ok(())
    }
}
