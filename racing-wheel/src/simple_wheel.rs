use crate::misc::bitflags;
use usb_device::{bus::UsbBus, prelude::UsbError};
use usb_hid_device::{
    hid::ReportWriter,
    hid_device::{HIDDeviceType, HIDReport, HIDReportIn, ReportID, ReportType},
};

pub struct SimpleWheel {
    pub report: SimpleWheelReport,
}

impl SimpleWheel {
    pub fn new() -> Self {
        Self {
            report: SimpleWheelReport::default(),
        }
    }
}

impl HIDDeviceType for SimpleWheel {
    fn descriptor() -> &'static [u8] {
        SIMPLE_WHEEL_DESCRIPTOR
    }

    fn send_input_reports<B: UsbBus>(&mut self, writer: ReportWriter<B>) -> Result<(), UsbError> {
        writer.write_report(self.report.clone())?;
        Ok(())
    }
}

// Simple wheel report
#[derive(Default, Clone)]
pub struct SimpleWheelReport {
    pub buttons: [bool; 8],
    pub throttle: u16,
    pub accelerator: u16,
    pub brake: u16,
    pub steering: u16,
}

impl HIDReport for SimpleWheelReport {
    const ID: ReportID = ReportID(ReportType::Input, 0x01);
}

impl HIDReportIn<9> for SimpleWheelReport {
    fn report_bytes(&self) -> [u8; 9] {
        [
            self.throttle.to_le_bytes()[0],
            self.throttle.to_le_bytes()[1],
            self.accelerator.to_le_bytes()[0],
            self.accelerator.to_le_bytes()[1],
            self.brake.to_le_bytes()[0],
            self.brake.to_le_bytes()[1],
            self.steering.to_le_bytes()[0],
            self.steering.to_le_bytes()[1],
            bitflags(&self.buttons),
        ]
    }
}

// A basic wheel descriptor for testing.
const SIMPLE_WHEEL_DESCRIPTOR: &[u8] = &[
    0x05, 0x01, // Usage Page (Generic Desktop Ctrls)
    0x09, 0x04, // Usage (Joystick)
    0xA1, 0x01, // Collection (Application)
    0x05, 0x02, // USAGE_PAGE (Simulation Controls)
    0x16, 0x01, 0x80, // LOGICAL_MINIMUM (-32767)
    0x26, 0xFF, 0x7F, // LOGICAL_MAXIMUM (+32767)
    0x75, 0x10, // REPORT_SIZE (16)
    0x95, 0x04, // REPORT_COUNT (simulationCount)
    0xA1, 0x00, // COLLECTION (Physical)
    0x09, 0xBB, // USAGE (Throttle)
    0x09, 0xC4, // USAGE (Accelerator)
    0x09, 0xC5, // USAGE (Brake)
    0x09, 0xC8, // USAGE (Steering)
    0x81, 0x02, // INPUT (Data,Var,Abs)
    0xc0, // END_COLLECTION (Physical)
    0x05, 0x09, //   Usage Page (Button)
    0x19, 0x01, //   Usage Minimum (0x01)
    0x29, 0x08, //   Usage Maximum (0x08)
    0x15, 0x00, //   Logical Minimum (0)
    0x25, 0x01, //   Logical Maximum (1)
    0x75, 0x01, //   Report Size (1)
    0x95, 0x08, //   Report Count (8)
    0x81, 0x02, //   Input (Data,Var,Abs,No Wrap,Linear,Preferred State,No Null Position)
    0xC0, // End Collection
];
