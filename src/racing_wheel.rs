use crate::{
    descriptor::RACING_WHEEL_DESCRIPTOR, hid::{HIDDeviceType, HIDReport, ReportID}, reports::{BlockLoadStatus, CreateNewEffectReport, PIDBlockLoadReport, PIDPoolReport}
};
use usb_device::{bus::UsbBus, class::ControlIn, UsbError};

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
        xfer: ControlIn<B>,
    ) -> Result<(), UsbError> {
        return match report_id {
            PIDBlockLoadReport::ID => xfer.accept_with(
                &PIDBlockLoadReport {
                    effect_block_index: (self.n_effect_blocks + 1) as u8,
                    block_load_status: BlockLoadStatus::Success,
                }
                .report_bytes(),
            ),
            PIDPoolReport::ID => xfer.accept_with(
                &PIDPoolReport {
                    ram_pool_size: 0x0000_00FF,
                    simultaneous_effects_max: 0x01,
                    device_managed_pool: true,
                    shared_parameter_blocks: false,
                }
                .report_bytes(),
            ),
            _ => Ok(()),
        };
    }

    fn set_report_request<B: UsbBus>(
        &mut self,
        report_id: ReportID,
        xfer: usb_device::class::ControlOut<B>,
    ) -> Result<(), UsbError> {
        return match report_id {
            CreateNewEffectReport::ID => {
                let _ = xfer.data();
                xfer.accept()
            }
            _ => Ok(()),
        };
    }
}
