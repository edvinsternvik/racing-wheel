#![no_std]
#![no_main]

mod pedals;

use cortex_m::asm::delay;
use cortex_m_rt::entry;
use panic_halt as _;
use pedals::Pedals;
use stm32f1xx_hal::adc::Adc;
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

    // Setup pins
    let mut gpioa = dp.GPIOA.split();

    let mut adc_throttle = Adc::adc1(dp.ADC1, clocks);
    let mut adc_brake = Adc::adc2(dp.ADC2, clocks);
    let mut analog_throttle_pin = gpioa.pa0.into_analog(&mut gpioa.crl);
    let mut analog_brake_pin = gpioa.pa1.into_analog(&mut gpioa.crl);

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

        if report_timer.wait().is_ok() {
            let throttle_raw: u16 = adc_throttle.read(&mut analog_throttle_pin).unwrap();
            let brake_raw: u16 = adc_brake.read(&mut analog_brake_pin).unwrap();

            let throttle = throttle_raw as f32 / adc_throttle.max_sample() as f32;
            let brake = brake_raw as f32 / adc_brake.max_sample() as f32;

            pedals.get_device_mut().set_throttle(throttle);
            pedals.get_device_mut().set_brake(brake);

            pedals.send_input_reports();
        }
    }
}
