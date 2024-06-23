use crate::hid_device::*;
use core::convert::TryInto;
use usb_device::{
    class_prelude::*,
    control::{Recipient, Request, RequestType},
    UsbDirection,
};

const USB_CLASS_HID: u8 = 0x03;
const HID_SPEC_VERSION: u16 = 0x01_11; // 01.11 in BCD
const MAX_PACKET_SIZE: usize = 64;

pub struct HID<'a, D: HIDDeviceType, B: UsbBus> {
    interface_number: InterfaceNumber,
    endpoint_in: EndpointIn<'a, B>,
    endpoint_out: EndpointOut<'a, B>,
    device: D,
}

impl<'a, D: HIDDeviceType, B: UsbBus> HID<'a, D, B> {
    pub fn new(alloc: &'a UsbBusAllocator<B>, device: D) -> HID<'a, D, B> {
        HID {
            interface_number: alloc.interface(),
            endpoint_in: alloc.interrupt(MAX_PACKET_SIZE as u16, 1),
            endpoint_out: alloc.interrupt(MAX_PACKET_SIZE as u16, 1),
            device,
        }
    }

    pub fn send_input_reports(&mut self) {
        let _ = self
            .device
            .send_input_reports(ReportWriter(&self.endpoint_in));
    }
}

pub struct GetReportInWriter<'a, 'p, 'r, B: UsbBus>(ControlIn<'a, 'p, 'r, B>);
impl<'a, 'p, 'r, B: UsbBus> GetReportInWriter<'a, 'p, 'r, B> {
    pub fn accept<const N: usize>(self, report: impl HIDReportIn<N>) -> Result<(), UsbError> {
        let data = report.report_bytes();
        self.0.accept_with(&data)?;
        Ok(())
    }
}
pub struct ReportWriter<'a, B: UsbBus>(&'a EndpointIn<'a, B>);
impl<'a, 'p, 'r, B: UsbBus> ReportWriter<'a, B> {
    pub fn write_report<const N: usize>(
        &self,
        report: impl HIDReportIn<N>,
    ) -> Result<(), UsbError> {
        let data = report.report_bytes();
        self.0.write(&data)?;
        Ok(())
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
        if request.direction != UsbDirection::Out
            || request.recipient != Recipient::Interface
            || request.index != u8::from(self.interface_number) as u16
        {
            return;
        }

        match (request.request_type, request.request) {
            (RequestType::Class, HIDRequest::SET_REPORT) => {
                let report_type = request.value.to_le_bytes()[1].try_into().unwrap();
                let report_id = request.value.to_le_bytes()[0];

                let report_identifier = ReportID(report_type, report_id);
                match self
                    .device
                    .report_request_out(report_identifier, xfer.data())
                    .unwrap_or_default()
                {
                    Some(true) => {
                        let _ = xfer.accept();
                    }
                    Some(false) => {
                        let _ = xfer.reject();
                    }
                    None => {
                        let _ = xfer.reject();
                    }
                };
            }
            _ => {}
        }
    }

    fn control_in(&mut self, xfer: ControlIn<B>) {
        let request = xfer.request();
        if request.direction != UsbDirection::In
            || request.recipient != Recipient::Interface
            || request.index != u8::from(self.interface_number) as u16
        {
            return;
        }

        match (request.request_type, request.request) {
            (RequestType::Standard, Request::GET_DESCRIPTOR) => {
                let descriptor_type = request.value.to_le_bytes()[1];

                if descriptor_type == ClassDescriptorType::REPORT as u8 {
                    let _ = xfer.accept_with_static(D::descriptor());
                }
            }
            (RequestType::Class, HIDRequest::GET_REPORT) => {
                let report_type = request.value.to_le_bytes()[1].try_into().unwrap();
                let report_id = request.value.to_le_bytes()[0];

                let report_identifier = ReportID(report_type, report_id);
                let _ = self
                    .device
                    .get_report_request(report_identifier, GetReportInWriter(xfer));
            }
            _ => {}
        }
    }

    fn endpoint_out(&mut self, addr: EndpointAddress) {
        if addr != self.endpoint_out.address() {
            return;
        }

        let mut buffer = [0; MAX_PACKET_SIZE];
        match self.endpoint_out.read(&mut buffer) {
            Ok(bytes_received) if bytes_received > 1 => {
                let _ = self
                    .device
                    .report_request_out(ReportID(ReportType::Output, buffer[0]), &buffer);
            }
            _ => {}
        }
    }
}
