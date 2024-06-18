use crate::{
    descriptor::RACING_WHEEL_DESCRIPTOR,
    hid::{GetReportInWriter, HIDDeviceType, HIDReportIn, HIDReportOut, ReportID, ReportWriter},
    reports::{
        BlockLoadStatus, CreateNewEffectReport, PIDBlockFreeReport, PIDBlockLoadReport,
        PIDPoolReport, PIDStateReport,
    },
};
use usb_device::{bus::UsbBus, UsbError};

pub struct RacingWheel {
    n_effect_blocks: usize,
}

impl RacingWheel {
    pub fn new() -> Self {
        RacingWheel { n_effect_blocks: 0 }
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
                self.n_effect_blocks += 1;
                writer.accept(PIDBlockLoadReport {
                    effect_block_index: self.n_effect_blocks as u8,
                    block_load_status: BlockLoadStatus::Success,
                    ram_pool_available: 0x00FF,
                })
            }
            PIDPoolReport::ID => writer.accept(PIDPoolReport {
                ram_pool_size: 0x00FF,
                simultaneous_effects_max: 0x01,
                device_managed_pool: true,
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
            CreateNewEffectReport::ID => Ok(Some(true)),
            PIDBlockFreeReport::ID => {
                let _ = PIDBlockFreeReport::into_report(data);
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
