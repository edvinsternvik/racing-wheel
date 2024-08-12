use fixed_num::Fixed16;
use usb_device::{bus::UsbBus, UsbError};
use usb_hid_device::{
    hid::ReportWriter,
    hid_device::{HIDDeviceType, HIDReport, HIDReportIn, ReportID, ReportType},
};

pub type PedalSignal = Fixed16<10_000>;

pub struct Pedals {
    pub report: PedalsReport,
}

impl Pedals {
    pub fn new() -> Self {
        Self {
            report: PedalsReport::default(),
        }
    }

    pub fn set_throttle(&mut self, throttle: PedalSignal) {
        self.report.throttle = throttle;
    }

    pub fn set_brake(&mut self, brake: PedalSignal) {
        self.report.brake = brake;
    }
}

impl HIDDeviceType for Pedals {
    fn descriptor() -> &'static [u8] {
        PEDALS_DESCRIPTOR
    }

    fn send_input_reports<B: UsbBus>(&mut self, writer: ReportWriter<B>) -> Result<(), UsbError> {
        writer.write_report(self.report.clone())?;
        Ok(())
    }
}

// Pedals Report
#[derive(Default, Clone)]
pub struct PedalsReport {
    pub throttle: PedalSignal,
    pub brake: PedalSignal,
}

impl HIDReport for PedalsReport {
    const ID: ReportID = ReportID(ReportType::Input, 0x01);
}

impl HIDReportIn<4> for PedalsReport {
    fn report_bytes(&self) -> [u8; 4] {
        [
            self.throttle.value().to_le_bytes()[0],
            self.throttle.value().to_le_bytes()[1],
            self.brake.value().to_le_bytes()[0],
            self.brake.value().to_le_bytes()[1],
        ]
    }
}

// HID descriptor
#[rustfmt::skip]
const PEDALS_DESCRIPTOR: &[u8] = &[
    0x05, 0x01,        // Usage Page (Generic Desktop Ctrls)
    0x09, 0x04,        // Usage (Joystick)
    0xA1, 0x01,        // Collection (Application)
    0x05, 0x02,        //   USAGE_PAGE (Simulation Controls)
    0x15, 0x00,        //   Logical Minimum (0)
    0x26, 0x10, 0x27,  //   Logical Maximum (10000)
    0x35, 0x00,        //   Physical Minimum (0)
    0x46, 0x10, 0x27,  //   Physical Maximum (10000)
    0x75, 0x10,        //   REPORT_SIZE (16)
    0x95, 0x02,        //   REPORT_COUNT (2)
    0xA1, 0x00,        //   COLLECTION (Physical)
    0x09, 0xBB,        //     USAGE (Throttle)
    0x09, 0xC5,        //     USAGE (Brake)
    0x81, 0x02,        //     INPUT (Data,Var,Abs)
    0xc0,              //   END_COLLECTION (Physical)
    0xC0,              // End Collection
];
