#![no_std]
#![no_main]

mod descriptor;
mod hid;
mod hid_device;
mod racing_wheel;
mod reports;

use crate::racing_wheel::RacingWheel;
use cortex_m::asm::delay;
use cortex_m_rt::entry;
use hid::HID;
use panic_halt as _;
//use panic_abort as _;
use stm32f1xx_hal::adc::Adc;
use stm32f1xx_hal::pac::Peripherals as HALPeripherals;
use stm32f1xx_hal::prelude::*;
use stm32f1xx_hal::usb::{Peripheral, UsbBus};
use usb_device::device::{UsbDeviceBuilder, UsbVidPid};

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

    assert!(clocks.usbclk_valid());

    // Setup buttons and analog input
    let mut gpioa = dp.GPIOA.split();

    let mut adc = Adc::adc1(dp.ADC1, clocks);
    let mut analog_x_pin = gpioa.pa0.into_analog(&mut gpioa.crl);
    let mut analog_y_pin = gpioa.pa1.into_analog(&mut gpioa.crl);

    let button_a = gpioa.pa2.into_pull_down_input(&mut gpioa.crl);
    let button_b = gpioa.pa3.into_pull_down_input(&mut gpioa.crl);

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

    let mut racing_wheel = HID::new(&usb_bus, RacingWheel::new());

    let mut usb_device = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0xF055, 0x5555))
        .manufacturer("Edvin")
        .product("PC Racing Wheel")
        .serial_number("RACINGWHEEL")
        .build();

    // Setup report timer
    let mut report_timer = dp.TIM2.counter_us(&clocks);
    report_timer.start(10.millis()).unwrap();

    // Poll USB and send state reports
    loop {
        usb_device.poll(&mut [&mut racing_wheel]);

        if report_timer.wait().is_ok() {
            let x_raw: u16 = adc.read(&mut analog_x_pin).unwrap();
            let y_raw: u16 = adc.read(&mut analog_y_pin).unwrap();

            let _x = (-(x_raw as i32 / 16) + 127) as i8;
            let _y = -((y_raw as i32 / 16) - 127) as i8;
            let _buttons: u8 = 0
                + if button_a.is_high() { 1 << 4 } else { 0 }
                + if button_b.is_high() { 1 << 5 } else { 0 };

            racing_wheel.send_input_reports();
        }
    }
}
