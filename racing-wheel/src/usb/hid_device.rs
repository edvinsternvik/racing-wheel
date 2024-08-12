use core::convert::TryFrom;
use usb_device::{bus::UsbBus, UsbError};
use crate::usb::hid::{GetReportInWriter, ReportWriter};

pub trait HIDDeviceType {
    fn descriptor() -> &'static [u8];
    fn get_report_request<B: UsbBus>(
        &mut self,
        report_id: ReportID,
        writer: GetReportInWriter<B>,
    ) -> Result<(), UsbError> {
        let (_, _) = (report_id, writer);
        Ok(())
    }
    fn report_request_out(
        &mut self,
        report_id: ReportID,
        data: &[u8],
    ) -> Result<Option<bool>, ()> {
        let (_, _) = (report_id, data);
        Ok(None)
    }
    fn send_input_reports<B: UsbBus>(&mut self, writer: ReportWriter<B>) -> Result<(), UsbError> {
        let _ = writer;
        Ok(())
    }
}

#[derive(PartialEq)]
pub struct ReportID(pub ReportType, pub u8);

pub trait HIDReport {
    const ID: ReportID;
}

pub trait HIDReportIn<const N: usize>
where
    Self: HIDReport
{
    // Serialize a report coming from the device, to be sent to the host.
    fn report_bytes(&self) -> [u8; N];
}

pub trait HIDReportOut
where
    Self: HIDReport + Sized,
{
    // Deserialize a report coming from the host, which has been read by the device.
    fn into_report(bytes: &[u8]) -> Option<Self>;
}

// Serializes and deserializes the report to RAM. This is done almost the same as for
// HIDReportIn/Out, except that the Effect Block Index and ROM flag(if it exists) is not
// serialized/deserialized.
pub trait HIDReportRAM<const N: usize>
where
    Self: HIDReport + Sized,
{
    // The size that the report takes up in RAM.
    const RAM_SIZE: usize = N;

    // Deserialize the report from a slice of the RAM pool starting at the address of the report.
    fn from_ram(ram: &[u8], effect_block_index: u8) -> Option<Self>;

    // Serialize the report to the format it will be stored in RAM.
    fn to_ram(&self) -> [u8; N];
}

#[derive(PartialEq)]
pub enum ReportType {
    Input = 0x01,
    Output = 0x02,
    Feature = 0x03,
}

impl TryFrom<u8> for ReportType {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x01 => Ok(ReportType::Input),
            0x02 => Ok(ReportType::Output),
            0x03 => Ok(ReportType::Feature),
            _ => Err(()),
        }
    }
}

