#![no_std]
#![no_main]

mod descriptor;
mod hid;
mod hid_device;
mod misc;
mod racing_wheel;
mod ram_pool;
mod reports;
mod simple_wheel;

use crate::hid::HID;
use crate::racing_wheel::RacingWheel;
use cortex_m::asm::delay;
use cortex_m_rt::entry;
use panic_halt as _;
use stm32f1xx_hal::adc::Adc;
use stm32f1xx_hal::gpio::*;
use stm32f1xx_hal::pac::Peripherals as HALPeripherals;
use stm32f1xx_hal::prelude::*;
use stm32f1xx_hal::usb::{Peripheral, UsbBus};
use usb_device::device::{UsbDeviceBuilder, UsbVidPid};

#[entry]
fn main() -> ! {
    let dp = HALPeripherals::take().unwrap();

    // Setup rotary encoder
    dp.RCC.apb1enr.write(|w| w.tim4en().set_bit());

    let mut gpiob = dp.GPIOB.split();
    gpiob.pb6.into_floating_input(&mut gpiob.crl);
    gpiob.pb7.into_floating_input(&mut gpiob.crl);

    dp.TIM4.smcr.write(|w| w.sms().encoder_mode_3());
    dp.TIM4.arr.write(|w| w.arr().variant(0xFF_FF));
    dp.TIM4.ccmr1_input().write(|w| w.cc1s().ti1());
    dp.TIM4.ccmr1_input().write(|w| w.cc2s().ti2());
    dp.TIM4.cr1.write(|w| w.cen().enabled());

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
    let mut analog_throttle_pin = gpioa.pa0.into_analog(&mut gpioa.crl);

    let button_a = gpioa.pa2.into_pull_down_input(&mut gpioa.crl);
    let button_b = gpioa.pa3.into_pull_down_input(&mut gpioa.crl);

    // LEDs
   let mut led_pins = [
        gpiob.pb5.into_push_pull_output(&mut gpiob.crl).erase(),
        //gpiob.pb6.into_push_pull_output(&mut gpiob.crl).erase(),
        //gpiob.pb7.into_push_pull_output(&mut gpiob.crl).erase(),
        //gpiob.pb8.into_push_pull_output(&mut gpiob.crh).erase(),
        //gpiob.pb9.into_push_pull_output(&mut gpiob.crh).erase(),
    ];

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
            let steering = dp.TIM4.cnt.read().cnt().bits() as i16;
            let throttle_raw: u16 = adc.read(&mut analog_throttle_pin).unwrap();
            let mut buttons = [false; 8];
            buttons[0] = button_a.is_high();
            buttons[1] = button_b.is_high();

            racing_wheel
                .get_device_mut()
                .set_throttle((-(throttle_raw as i32) + 2047) as i16 * 16);
            racing_wheel
                .get_device_mut()
                .set_steering(steering);
            racing_wheel.get_device_mut().set_buttons(buttons);

            let ffb = racing_wheel.get_device().get_force_feedback();
            const FFB_MAX: i32 = 10_000;
            let n_leds = (ffb as i64 * led_pins.len() as i64) / (FFB_MAX as i64);
            for i in 0..led_pins.len() {
                if (i as i64) < n_leds {
                    led_pins[i].set_high();
                } else {
                    led_pins[i].set_low();
                }
            }

            racing_wheel.send_input_reports();
        }
    }
}
