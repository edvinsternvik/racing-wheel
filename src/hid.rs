use usb_device::{
    class_prelude::*,
    control::{Request, RequestType},
};

const USB_CLASS_HID: u8 = 0x03;
const HID_SPEC_VERSION: u16 = 0x01_11; // 01.11 in BCD

pub struct HID<'a, B: UsbBus> {
    interface_number: InterfaceNumber,
    endpoint_in: EndpointIn<'a, B>,
    endpoint_out: EndpointOut<'a, B>,
    descriptor: &'static [u8],
}

impl<'a, B: UsbBus> HID<'a, B> {
    pub fn new(alloc: &'a UsbBusAllocator<B>, descriptor: &'static [u8]) -> HID<'a, B> {
        HID {
            interface_number: alloc.interface(),
            endpoint_in: alloc.interrupt(64, 1),
            endpoint_out: alloc.interrupt(64, 1),
            descriptor,
        }
    }


    pub fn write_report(&mut self, report: &[u8]) -> Result<(), ()> {
        self.endpoint_in.write(&report).map(|_| ()).map_err(|_| ())
    }
}

pub struct ClassDescriptorType;
impl ClassDescriptorType {
    const HID: u8 = 0x21;
    const REPORT: u8 = 0x22;
    const _PHYSICAL: u8 = 0x23;
}

pub struct HIDRequest;
impl HIDRequest {
    const GET_REPORT: u8 = 0x01;
    const _GET_IDLE: u8 = 0x02;
    const _GET_PROTOCOL: u8 = 0x03;
    const _SET_REPORT: u8 = 0x09;
    const _SET_IDLE: u8 = 0x0A;
    const _SET_PROTOCOL: u8 = 0x0B;
}

impl<B: UsbBus> UsbClass<B> for HID<'_, B> {
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
        let descriptor_length: u16 = self.descriptor.len() as u16;

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
        let _ = xfer;
    }

    fn control_in(&mut self, xfer: ControlIn<B>) {
        let request = xfer.request();

        match (request.request_type, request.request) {
            (RequestType::Standard, Request::GET_DESCRIPTOR) => {
                let descriptor_type = request.value.to_le_bytes()[1];

                if descriptor_type == ClassDescriptorType::REPORT as u8 {
                    xfer.accept_with_static(self.descriptor).unwrap();
                }
            }
            (RequestType::Class, HIDRequest::GET_REPORT) => {}
            (_, _) => {}

        }
    }
}
