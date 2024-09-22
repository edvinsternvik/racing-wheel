#![no_std]
#![no_main]

mod hx711;
mod pedals;

use cortex_m::asm::delay;
use cortex_m_rt::entry;
use hx711::HX711;
use panic_halt as _;
use pedals::Pedals;
use stm32f1xx_hal::gpio::*;
use stm32f1xx_hal::pac::Peripherals as HALPeripherals;
use stm32f1xx_hal::prelude::*;
use stm32f1xx_hal::usb::{Peripheral, UsbBus};
use usb_device::device::{UsbDeviceBuilder, UsbVidPid};
use usb_hid_device::hid::HID;

#[entry]
fn main() -> ! {
    let dp = HALPeripherals::take().unwrap();

    // Setup clocks
    let mut flash = dp.FLASH.constrain();
    let rcc = dp.RCC.constrain();

    let clocks = rcc
        .cfgr
        .use_hse(8.MHz())
        .sysclk(48.MHz())
        .pclk1(24.MHz())
        .adcclk(2.MHz())
        .freeze(&mut flash.acr);

    // Setup load cell
    let mut gpioa = dp.GPIOA.split();

    let d_out = gpioa.pa6.into_floating_input(&mut gpioa.crl).erase();
    let pd_sck = gpioa.pa7.into_push_pull_output(&mut gpioa.crl).erase();
    let mut hx711 = HX711::new(d_out, pd_sck, clocks.sysclk().raw());

    // Setup USB
    let mut usb_dp = gpioa.pa12.into_push_pull_output(&mut gpioa.crh);
    usb_dp.set_low();
    delay(clocks.sysclk().raw() / 100);

    let usb_peripheral = Peripheral {
        usb: dp.USB,
        pin_dm: gpioa.pa11,
        pin_dp: usb_dp.into_floating_input(&mut gpioa.crh),
    };
    let usb_bus = UsbBus::new(usb_peripheral);

    let mut pedals = HID::new(&usb_bus, Pedals::new());

    let mut usb_device = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0xF055, 0x5556))
        .manufacturer("Edvin")
        .product("PC Racing Pedals")
        .serial_number("RACINGPEDALS")
        .build();

    // Setup report timer
    let mut report_timer = dp.TIM2.counter_us(&clocks);
    report_timer.start(10.millis()).unwrap();

    // Poll USB and send state reports
    loop {
        usb_device.poll(&mut [&mut pedals]);

        if hx711.data_available() {
            let brake = hx711.read_data();
            pedals.get_device_mut().set_brake(brake);
        }

        if report_timer.wait().is_ok() {
            pedals.send_input_reports();
        }
    }
}
