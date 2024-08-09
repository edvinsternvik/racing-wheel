#![no_std]
#![no_main]

mod misc;
mod motor;
mod racing_wheel;
mod simple_wheel;
mod usb;

use cortex_m::asm::delay;
use cortex_m_rt::entry;
use fixed_num::Frac16;
use motor::Motor;
use panic_halt as _;
use racing_wheel::RacingWheel;
use stm32f1xx_hal::adc::Adc;
use stm32f1xx_hal::gpio::*;
use stm32f1xx_hal::pac::Peripherals as HALPeripherals;
use stm32f1xx_hal::prelude::*;
use stm32f1xx_hal::timer::Tim3NoRemap;
use stm32f1xx_hal::usb::{Peripheral, UsbBus};
use usb::hid::HID;
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

    // Setup motor
    let mut gpioa = dp.GPIOA.split();
    let mut afio = dp.AFIO.constrain();

    let motor_enable_pin = gpioa.pa5.into_push_pull_output(&mut gpioa.crl);

    let reverse_pwm_pin = gpioa.pa6.into_alternate_push_pull(&mut gpioa.crl);
    let forward_pwm_pin = gpioa.pa7.into_alternate_push_pull(&mut gpioa.crl);
    let pwm_pins = (reverse_pwm_pin, forward_pwm_pin);
    let pwm = dp
        .TIM3
        .pwm_hz::<Tim3NoRemap, _, _>(pwm_pins, &mut afio.mapr, 8_000.Hz(), &clocks);
    let (pwm_reverse, pwm_forward) = pwm.split();
    let mut motor = Motor::new(motor_enable_pin.erase(), pwm_forward, pwm_reverse);

    // Setup buttons and analog input
    let mut adc_throttle = Adc::adc1(dp.ADC1, clocks);
    let mut adc_brake = Adc::adc2(dp.ADC2, clocks);
    let mut analog_throttle_pin = gpioa.pa0.into_analog(&mut gpioa.crl);
    let mut analog_brake_pin = gpioa.pa1.into_analog(&mut gpioa.crl);

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
            let steering_raw = dp.TIM4.cnt.read().cnt().bits() as i16;
            let throttle_raw: u16 = adc_throttle.read(&mut analog_throttle_pin).unwrap();
            let brake_raw: u16 = adc_brake.read(&mut analog_brake_pin).unwrap();
            let mut buttons = [false; 8];
            buttons[0] = button_a.is_high();
            buttons[1] = button_b.is_high();

            let steering = steering_raw.into();
            let throttle = Frac16::new(throttle_raw as i16, adc_throttle.max_sample() as i16);
            let brake = Frac16::new(brake_raw as i16, adc_brake.max_sample() as i16);

            let t_val = if throttle > brake { throttle } else { -brake };
            racing_wheel.get_device_mut().set_throttle(t_val.convert());
            racing_wheel.get_device_mut().set_steering(steering);
            racing_wheel.get_device_mut().set_buttons(buttons);

            let ffb = racing_wheel.get_device().get_force_feedback();
            racing_wheel.get_device_mut().advance(10);

            motor.set_speed(ffb);

            racing_wheel.send_input_reports();
        }
    }
}
