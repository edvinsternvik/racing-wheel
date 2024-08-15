#![no_std]
#![no_main]

mod config;
mod misc;
mod motor;
mod racing_wheel;
mod simple_wheel;

use config::Config;
use cortex_m::asm::delay;
use cortex_m_rt::entry;
use motor::Motor;
use panic_halt as _;
use racing_wheel::RacingWheel;
use stm32f1xx_hal::flash::{FlashSize, SectorSize};
use stm32f1xx_hal::gpio::*;
use stm32f1xx_hal::pac::Peripherals as HALPeripherals;
use stm32f1xx_hal::prelude::*;
use stm32f1xx_hal::timer::Tim3NoRemap;
use stm32f1xx_hal::usb::{Peripheral, UsbBus};
use usb_device::device::{UsbDeviceBuilder, UsbVidPid};
use usb_hid_device::hid::HID;

const ENCODER_TO_DEG: f32 = 360.0 / 2400.0;

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

    // Setup motor
    let mut gpioa = dp.GPIOA.split();
    let mut afio = dp.AFIO.constrain();

    let motor_enable_pin = gpioa.pa5.into_push_pull_output(&mut gpioa.crl);

    let reverse_pwm_pin = gpioa.pa7.into_alternate_push_pull(&mut gpioa.crl);
    let forward_pwm_pin = gpioa.pa6.into_alternate_push_pull(&mut gpioa.crl);
    let pwm_pins = (forward_pwm_pin, reverse_pwm_pin);
    let pwm = dp
        .TIM3
        .pwm_hz::<Tim3NoRemap, _, _>(pwm_pins, &mut afio.mapr, 8_000.Hz(), &clocks);
    let (pwm_forward, pwm_reverse) = pwm.split();
    let mut motor = Motor::new(motor_enable_pin.erase(), pwm_forward, pwm_reverse);

    // Setup buttons
    let button_a = gpiob.pb10.into_pull_down_input(&mut gpiob.crh);
    let button_b = gpiob.pb11.into_pull_down_input(&mut gpiob.crh);

    // Setup config
    let mut flash_writer = flash.writer(SectorSize::Sz1K, FlashSize::Sz128K);
    let config = Config::read_from_memory(&flash_writer);

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

    let mut racing_wheel = HID::new(&usb_bus, RacingWheel::new(config));

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

        if racing_wheel.get_device_mut().write_config_event() {
            racing_wheel
                .get_device()
                .get_config()
                .write_to_memory(&mut flash_writer);
        }

        if report_timer.wait().is_ok() {
            let steering_raw = dp.TIM4.cnt.read().cnt().bits() as i16;
            let steering = steering_raw as f32 * ENCODER_TO_DEG;
            let mut buttons = [false; 8];
            buttons[0] = button_a.is_high();
            buttons[1] = button_b.is_high();

            racing_wheel.get_device_mut().set_steering(steering);
            racing_wheel.get_device_mut().set_buttons(buttons);

            let ffb = racing_wheel.get_device().get_force_feedback();
            racing_wheel.get_device_mut().advance(10);

            let config = racing_wheel.get_device().get_config();
            motor.set_speed(ffb, config.motor_max, config.motor_deadband);

            racing_wheel.send_input_reports();
        }
    }
}
