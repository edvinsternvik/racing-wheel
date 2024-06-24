use crate::{
    descriptor::RACING_WHEEL_DESCRIPTOR,
    hid::{GetReportInWriter, ReportWriter},
    hid_device::{HIDDeviceType, HIDReport, HIDReportOut, HIDReportRAM, ReportID},
    misc::FixedSet,
    ram_pool::{effect_address, RAMPool},
    reports::{
        BlockLoadStatus, CreateNewEffectReport, CustomForceDataReport, DeviceControl,
        DeviceGainReport, EffectOperation, EffectOperationReport, EffectType, PIDBlockFreeReport,
        PIDBlockLoadReport, PIDDeviceControl, PIDPoolMoveReport, PIDPoolReport, PIDStateReport,
        RacingWheelReport, SetConditionReport, SetConstantForceReport, SetCustomForceReport,
        SetEffectReport, SetEnvelopeReport, SetPeriodicReport, SetRampForceReport,
    },
};
use usb_device::{bus::UsbBus, UsbError};

const RAM_POOL_SIZE: usize = 16384;
const MAX_EFFECTS: usize = 16;
const MAX_SIMULTANEOUS_EFFECTS: usize = 8;

pub struct RacingWheel {
    ram_pool: RAMPool<RAM_POOL_SIZE, MAX_EFFECTS>,
    next_effect: Option<CreateNewEffectReport>,
    running_effects: FixedSet<u8, MAX_SIMULTANEOUS_EFFECTS>,
    device_gain: u8,
    joystick_report: RacingWheelReport,
    pid_state_report: PIDStateReport,
}

impl RacingWheel {
    pub fn new() -> Self {
        RacingWheel {
            ram_pool: RAMPool::new(),
            next_effect: None,
            running_effects: FixedSet::new(),
            device_gain: 0,
            joystick_report: RacingWheelReport::default(),
            pid_state_report: PIDStateReport::default(),
        }
    }

    pub fn joystick_report_mut(&mut self) -> &mut RacingWheelReport {
        &mut self.joystick_report
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
                if let Some(next_effect) = self.next_effect {
                    self.next_effect = None;

                    if let Some(index) = self.ram_pool.new_effect(next_effect.effect_type) {
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
                let mut report = SetEffectReport::into_report(data).ok_or(())?;
                let parameter_ram_sizes = get_parameter_ram_sizes(report.effect_type);
                let address = self
                    .ram_pool
                    .allocate(parameter_ram_sizes.iter().sum())
                    .map_err(|_| ())?;

                report.type_specific_block_offset_instance_1 = address as u16;
                report.type_specific_block_offset_instance_2 =
                    (address + parameter_ram_sizes[0]) as u16;

                self.ram_pool
                    .write_report(&report, effect_address(report.effect_block_index))
                    .map_err(|_| ())?;
                Ok(Some(true))
            }
            SetEnvelopeReport::ID => {
                let report = SetEnvelopeReport::into_report(data).ok_or(())?;
                let address = self
                    .ram_pool
                    .get_type_specific_block_offsets(report.effect_block_index)?[1];

                self.ram_pool.write_report(&report, address)?;
                Ok(Some(true))
            }
            SetConditionReport::ID => {
                let report = SetConditionReport::into_report(data).ok_or(())?;
                let addresses = self
                    .ram_pool
                    .get_type_specific_block_offsets(report.effect_block_index)?;
                let address = *addresses
                    .get(report.parameter_block_offset as usize)
                    .ok_or(())?;

                self.ram_pool.write_report(&report, address)?;
                Ok(Some(true))
            }
            SetPeriodicReport::ID => {
                let report = SetPeriodicReport::into_report(data).ok_or(())?;
                let address = self
                    .ram_pool
                    .get_type_specific_block_offsets(report.effect_block_index)?[0];

                self.ram_pool.write_report(&report, address)?;
                Ok(Some(true))
            }
            SetConstantForceReport::ID => {
                let report = SetConstantForceReport::into_report(data).ok_or(())?;
                let address = self
                    .ram_pool
                    .get_type_specific_block_offsets(report.effect_block_index)?[0];

                self.ram_pool.write_report(&report, address)?;
                Ok(Some(true))
            }
            SetRampForceReport::ID => {
                let report = SetRampForceReport::into_report(data).ok_or(())?;
                let address = self
                    .ram_pool
                    .get_type_specific_block_offsets(report.effect_block_index)?[0];

                self.ram_pool.write_report(&report, address)?;
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
                let address = self
                    .ram_pool
                    .get_type_specific_block_offsets(report.effect_block_index)?[0];

                self.ram_pool.write_report(&report, address)?;
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
        writer.write_report(self.joystick_report.clone())?;
        writer.write_report(self.pid_state_report.clone())?;

        Ok(())
    }
}

fn get_parameter_ram_sizes(effect_type: EffectType) -> [usize; 2] {
    match effect_type {
        EffectType::ConstantForce => [
            SetConstantForceReport::RAM_SIZE,
            SetEnvelopeReport::RAM_SIZE,
        ],
        EffectType::Ramp => [SetRampForceReport::RAM_SIZE, SetEnvelopeReport::RAM_SIZE],
        EffectType::Square => [SetPeriodicReport::RAM_SIZE, SetEnvelopeReport::RAM_SIZE],
        EffectType::Sine => [SetPeriodicReport::RAM_SIZE, SetEnvelopeReport::RAM_SIZE],
        EffectType::Triangle => [SetPeriodicReport::RAM_SIZE, SetEnvelopeReport::RAM_SIZE],
        EffectType::SawtoothUp => [SetPeriodicReport::RAM_SIZE, SetEnvelopeReport::RAM_SIZE],
        EffectType::SawtoothDown => [SetPeriodicReport::RAM_SIZE, SetEnvelopeReport::RAM_SIZE],
        EffectType::Spring => [SetConditionReport::RAM_SIZE, SetConditionReport::RAM_SIZE],
        EffectType::Damper => [SetConditionReport::RAM_SIZE, SetConditionReport::RAM_SIZE],
        EffectType::Inertia => [SetConditionReport::RAM_SIZE, SetConditionReport::RAM_SIZE],
        EffectType::Friction => [SetConditionReport::RAM_SIZE, SetConditionReport::RAM_SIZE],
        EffectType::CustomForceData => [SetCustomForceReport::RAM_SIZE, 0],
    }
}
