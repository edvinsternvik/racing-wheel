use core::convert::{TryFrom, TryInto};
use usb_device::{
    class_prelude::*,
    control::{Request, RequestType},
};

const USB_CLASS_HID: u8 = 0x03;
const HID_SPEC_VERSION: u16 = 0x01_11; // 01.11 in BCD

pub trait HIDDeviceType {
    fn descriptor() -> &'static [u8];
    fn get_report_request<B: UsbBus>(
        &mut self,
        report_id: ReportID,
        xfer: ControlIn<B>,
    ) -> Result<(), UsbError> {
        let (_, _) = (report_id, xfer);
        Ok(())
    }
    fn set_report_request<B: UsbBus>(
        &mut self,
        report_id: ReportID,
        xfer: ControlOut<B>,
    ) -> Result<(), UsbError> {
        let (_, _) = (report_id, xfer);
        Ok(())
    }
}

pub struct HID<'a, D: HIDDeviceType, B: UsbBus> {
    interface_number: InterfaceNumber,
    pub endpoint_in: EndpointIn<'a, B>,
    endpoint_out: EndpointOut<'a, B>,
    device: D,
}

impl<'a, D: HIDDeviceType, B: UsbBus> HID<'a, D, B> {
    pub fn new(alloc: &'a UsbBusAllocator<B>, device: D) -> HID<'a, D, B> {
        HID {
            interface_number: alloc.interface(),
            endpoint_in: alloc.interrupt(64, 1),
            endpoint_out: alloc.interrupt(64, 1),
            device,
        }
    }

    //pub fn write_report<const N: usize>(&mut self, report: impl HIDReport<N>) -> Result<(), ()> {
    //    let data = report.report_bytes();
    //    self.endpoint_in.write(&data).map(|_| ()).map_err(|_| ())
    //}
}

#[derive(PartialEq)]
pub struct ReportID(pub ReportType, pub u8);

pub trait HIDReport<const N: usize> {
    const ID: ReportID;
    fn report_bytes(&self) -> [u8; N];
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

struct ClassDescriptorType;
impl ClassDescriptorType {
    const HID: u8 = 0x21;
    const REPORT: u8 = 0x22;
    const _PHYSICAL: u8 = 0x23;
}

struct HIDRequest;
impl HIDRequest {
    const GET_REPORT: u8 = 0x01;
    const _GET_IDLE: u8 = 0x02;
    const _GET_PROTOCOL: u8 = 0x03;
    const SET_REPORT: u8 = 0x09;
    const _SET_IDLE: u8 = 0x0A;
    const _SET_PROTOCOL: u8 = 0x0B;
}

impl<D: HIDDeviceType, B: UsbBus> UsbClass<B> for HID<'_, D, B> {
    fn get_configuration_descriptors(
        &self,
        writer: &mut DescriptorWriter,
    ) -> usb_device::Result<()> {
        // Interface descriptor
        writer.interface(self.interface_number, USB_CLASS_HID, 0, 0)?;

        // HID descriptor
        let country_code: u8 = 0;
        let num_descriptors: u8 = 1;
        let descriptor_type = ClassDescriptorType::REPORT;
        let descriptor_length: u16 = D::descriptor().len() as u16;

        writer.write(
            ClassDescriptorType::HID as u8,
            &[
                HID_SPEC_VERSION.to_le_bytes()[0],
                HID_SPEC_VERSION.to_le_bytes()[1],
                country_code,
                num_descriptors,
                descriptor_type as u8,
                descriptor_length.to_le_bytes()[0],
                descriptor_length.to_le_bytes()[1],
            ],
        )?;

        // Endpoint descriptors
        writer.endpoint(&self.endpoint_in)?;
        writer.endpoint(&self.endpoint_out)?;

        Ok(())
    }

    fn reset(&mut self) {}

    fn poll(&mut self) {}

    fn control_out(&mut self, xfer: ControlOut<B>) {
        let request = xfer.request();

        match (request.request_type, request.request) {
            (RequestType::Class, HIDRequest::SET_REPORT) => {
                let report_type = request.value.to_le_bytes()[1].try_into().unwrap();
                let report_id = request.value.to_le_bytes()[0];

                let report_identifier = ReportID(report_type, report_id);
                self.device
                    .set_report_request(report_identifier, xfer)
                    .unwrap();
            }
            _ => {}
        }
    }

    fn control_in(&mut self, xfer: ControlIn<B>) {
        let request = xfer.request();

        match (request.request_type, request.request) {
            (RequestType::Standard, Request::GET_DESCRIPTOR) => {
                let descriptor_type = request.value.to_le_bytes()[1];

                if descriptor_type == ClassDescriptorType::REPORT as u8 {
                    xfer.accept_with_static(D::descriptor()).unwrap();
                }
            }
            (RequestType::Class, HIDRequest::GET_REPORT) => {
                let report_type = request.value.to_le_bytes()[1].try_into().unwrap();
                let report_id = request.value.to_le_bytes()[0];

                let report_identifier = ReportID(report_type, report_id);
                self.device
                    .get_report_request(report_identifier, xfer)
                    .unwrap();
            }
            _ => {}
        }
    }

    fn endpoint_out(&mut self, addr: EndpointAddress) {
        if addr != self.endpoint_out.address() {
            return;
        }

        let mut buffer = [0; 1024];
        match self.endpoint_out.read(&mut buffer) {
            Ok(_bytes_received) => {}
            Err(_) => {}
        }
    }
}
