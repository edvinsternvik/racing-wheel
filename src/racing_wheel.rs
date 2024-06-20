use crate::{
    descriptor::RACING_WHEEL_DESCRIPTOR,
    hid::{GetReportInWriter, ReportWriter},
    hid_device::{HIDDeviceType, HIDReport, HIDReportOut, ReportID},
    reports::{
        BlockLoadStatus, CreateNewEffectReport, CustomForceDataReport, DeviceGainReport,
        EffectOperationReport, PIDBlockFreeReport, PIDBlockLoadReport, PIDDeviceControl,
        PIDPoolReport, PIDStateReport, SetConditionReport, SetConstantForceReport,
        SetCustomForceReport, SetEffectReport, SetEnvelopeReport, SetPeriodicReport,
        SetRampForceReport,
    },
};
use usb_device::{bus::UsbBus, UsbError};

struct RAMPool<const N: usize> {
    buffer: [u8; N],
    n_effect_blocks: usize,
}

impl<const N: usize> RAMPool<N> {
    fn new() -> Self {
        Self {
            buffer: [0; N],
            n_effect_blocks: 0,
        }
    }
}

pub struct RacingWheel {
    ram_pool: RAMPool<4096>,
}

impl RacingWheel {
    pub fn new() -> Self {
        RacingWheel {
            ram_pool: RAMPool::new(),
        }
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
                self.ram_pool.n_effect_blocks += 1;
                writer.accept(PIDBlockLoadReport {
                    effect_block_index: self.ram_pool.n_effect_blocks as u8,
                    block_load_status: BlockLoadStatus::Success,
                    ram_pool_available: 0x00FF,
                })
            }
            PIDPoolReport::ID => writer.accept(PIDPoolReport {
                ram_pool_size: self.ram_pool.buffer.len() as u16,
                simultaneous_effects_max: 0x01,
                param_block_size_set_effect: todo!(),
                param_block_size_set_envelope: todo!(),
                param_block_size_set_condition: todo!(),
                param_block_size_set_periodic: todo!(),
                param_block_size_set_constant_force: todo!(),
                param_block_size_set_ramp_force: todo!(),
                param_block_size_set_custom_force: todo!(),
                device_managed_pool: false,
                shared_parameter_blocks: false,
            }),
            _ => Ok(()),
        }
    }

    fn report_request_out(
        &mut self,
        report_id: ReportID,
        data: &[u8],
    ) -> Result<Option<bool>, UsbError> {
        match report_id {
            SetEffectReport::ID => {
                let _ = SetEffectReport::into_report(data).ok_or(UsbError::ParseError)?;
                Ok(Some(true))
            }
            SetEnvelopeReport::ID => {
                let _ = SetEnvelopeReport::into_report(data).ok_or(UsbError::ParseError)?;
                Ok(Some(true))
            }
            SetConditionReport::ID => {
                let _ = SetConditionReport::into_report(data).ok_or(UsbError::ParseError)?;
                Ok(Some(true))
            }
            SetPeriodicReport::ID => {
                let _ = SetPeriodicReport::into_report(data).ok_or(UsbError::ParseError)?;
                Ok(Some(true))
            }
            SetConstantForceReport::ID => {
                let _ = SetConstantForceReport::into_report(data).ok_or(UsbError::ParseError)?;
                Ok(Some(true))
            }
            SetRampForceReport::ID => {
                let _ = SetRampForceReport::into_report(data).ok_or(UsbError::ParseError)?;
                Ok(Some(true))
            }
            CustomForceDataReport::ID => {
                let _ = CustomForceDataReport::into_report(data).ok_or(UsbError::ParseError)?;
                Ok(Some(true))
            }
            EffectOperationReport::ID => {
                let _ = EffectOperationReport::into_report(data).ok_or(UsbError::ParseError)?;
                Ok(Some(true))
            }
            PIDBlockFreeReport::ID => {
                let _ = PIDBlockFreeReport::into_report(data).ok_or(UsbError::ParseError)?;
                Ok(Some(true))
            }
            PIDDeviceControl::ID => {
                let _ = PIDDeviceControl::into_report(data).ok_or(UsbError::ParseError)?;
                Ok(Some(true))
            }
            DeviceGainReport::ID => {
                let _ = DeviceGainReport::into_report(data).ok_or(UsbError::ParseError)?;
                Ok(Some(true))
            }
            SetCustomForceReport::ID => {
                let _ = SetCustomForceReport::into_report(data).ok_or(UsbError::ParseError)?;
                Ok(Some(true))
            }
            CreateNewEffectReport::ID => {
                let _ = CreateNewEffectReport::into_report(data).ok_or(UsbError::ParseError)?;
                Ok(Some(true))
            }
            _ => Ok(None),
        }
    }

    fn send_input_reports<B: UsbBus>(&mut self, writer: ReportWriter<B>) -> Result<(), UsbError> {
        //writer.write_report(JoystickReport {
        //    buttons: [false; 8],
        //    joystick_x: 0,
        //    joystick_y: 0,
        //})?;

        writer.write_report(PIDStateReport {
            device_paused: false,
            actuators_enabled: false,
            safety_switch: false,
            actuators_override_switch: false,
            actuator_power: false,
            effect_playing: false,
            effect_block_index: 0,
        })?;

        Ok(())
    }
}
