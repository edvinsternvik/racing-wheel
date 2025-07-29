use config::{config::Config, control::WheelDeviceControl};
use hidapi::{HidApi, HidDevice};
use std::slice::Iter;
use force_feedback::reports::RacingWheelState;

const USB_VID: u16 = 0xF055;
const USB_PID: u16 = 0x5555;
const CONFIG_REPORT_ID: u8 = 0x04;
const WHEEL_DEVICE_CONTROL_REPORT_ID: u8 = 0x05;

enum Error {
    UsbHidError,
    DeviceError,
    SendError,
    ReadError,
    InvalidArgument,
    NotEnoughArguments,
    ParseError,
}

fn send_config(device: &HidDevice, config: Config) -> Result<(), Error> {
    let buf = config.into_bytes(CONFIG_REPORT_ID);
    device.send_feature_report(&buf).or(Err(Error::SendError))
}

fn read_config(device: &HidDevice) -> Result<Config, Error> {
    let mut buf = [0; 63];
    buf[0] = CONFIG_REPORT_ID;

    let bytes_read = device
        .get_feature_report(&mut buf)
        .or(Err(Error::ReadError))?;

    if bytes_read != buf.len() {
        println!("{}", bytes_read);
        return Err(Error::ReadError);
    }

    Config::from_bytes(&buf[1..]).ok_or(Error::ParseError)
}

fn set_option(mut args: Iter<String>) -> Result<(), Error> {
    let hid = HidApi::new().or(Err(Error::UsbHidError))?;
    let device = hid.open(USB_VID, USB_PID).or(Err(Error::DeviceError))?;
    let mut config = read_config(&device)?;

    let option = args.next().ok_or(Error::NotEnoughArguments)?;
    let value = args.next().ok_or(Error::NotEnoughArguments)?;

    match option.as_str() {
        "gain" => config.gain = value.parse().or(Err(Error::ParseError))?,
        "expo" => config.expo = value.parse().or(Err(Error::ParseError))?,
        "derivative_smoothing" => {
            config.derivative_smoothing = value.parse().or(Err(Error::ParseError))?
        }
        "max_rotation" => config.max_rotation = value.parse().or(Err(Error::ParseError))?,
        "spring_gain" => config.spring_gain = value.parse().or(Err(Error::ParseError))?,
        "spring_coefficient" => {
            config.spring_coefficient = value.parse().or(Err(Error::ParseError))?
        }
        "spring_saturation" => {
            config.spring_saturation = value.parse().or(Err(Error::ParseError))?
        }
        "spring_deadband" => config.spring_deadband = value.parse().or(Err(Error::ParseError))?,
        "damper_gain" => config.damper_gain = value.parse().or(Err(Error::ParseError))?,
        "damper_coefficient" => {
            config.damper_coefficient = value.parse().or(Err(Error::ParseError))?
        }
        "damper_saturation" => {
            config.damper_saturation = value.parse().or(Err(Error::ParseError))?
        }
        "damper_deadband" => config.damper_deadband = value.parse().or(Err(Error::ParseError))?,
        "motor_min" => config.motor_min = value.parse().or(Err(Error::ParseError))?,
        "motor_max" => config.motor_max = value.parse().or(Err(Error::ParseError))?,
        "motor_deadband" => config.motor_deadband = value.parse().or(Err(Error::ParseError))?,
        "motor_frequency_hz" => {
            config.motor_frequency_hz = value.parse().or(Err(Error::ParseError))?
        }
        "update_frequency_hz" => {
            config.update_frequency_hz = value.parse().or(Err(Error::ParseError))?
        }
        _ => return Err(Error::InvalidArgument),
    }

    send_config(&device, config)?;

    Ok(())
}

fn send_control_command(mut args: Iter<String>) -> Result<(), Error> {
    let hid = HidApi::new().or(Err(Error::UsbHidError))?;
    let device = hid.open(USB_VID, USB_PID).or(Err(Error::DeviceError))?;

    let config = read_config(&device)?;
    println!("{:?}", config);

    let command = args.next().ok_or(Error::NotEnoughArguments)?;
    let command_id = match command.as_str() {
        "reboot" => Ok(WheelDeviceControl::Reboot as u8),
        "reset_rotation" => Ok(WheelDeviceControl::ResetRotation as u8),
        "write_config" => Ok(WheelDeviceControl::WriteConfig as u8),
        _ => Err(Error::ParseError),
    }?;
    let buf = [WHEEL_DEVICE_CONTROL_REPORT_ID, command_id];

    device.send_feature_report(&buf).or(Err(Error::SendError))?;

    Ok(())
}

fn read_config_action() -> Result<(), Error> {
    let hid = HidApi::new().or(Err(Error::UsbHidError))?;
    let device = hid.open(USB_VID, USB_PID).or(Err(Error::DeviceError))?;

    let config = read_config(&device)?;
    println!("{:?}", config);

    Ok(())
}

fn print_help() -> Result<(), Error> {
    println!(
    r#"
    USAGE:
        configurator COMMAND

    COMMAND:
        config CONFIG_COMMAND       Set some configuration option, see CONFIG_COMMAND for the
                                    list of config commands.
        control CONTROL_COMMAND     Perform some control action, see CONTROL_COMMAND for the list
                                    of control commands.
        read_config                 Read the current configuration options.
        help                        Display this help page.

    CONFIG_COMMAND:
        gain <g>                    Set the motor gain.
        expo <e>                    Set the motor expo.
        derivative_smoothing <ds>   Set the derivative smoothing.
        max_rotation <mr>           Set the max rotation.
        spring_gain <sg>            Set the spring gain.
        spring_coefficient <sc>     Set the spring coefficient.
        spring_saturation <ss>      Set the spring saturation.
        spring_deadband <sd>        Set the spring deadband.
        damper_gain <dg>            Set the damper gain.
        damper_coefficient <dc>     Set the damper coefficient.
        damper_saturation <ds>      Set the damper saturation.
        damper_deadband <dd>        Set the damper deadband.
        motor_min <mm>              Set the motor min.
        motor_max <mm>              Set the motor max.
        motor_deadband <md>         Set the motor deadband.
        motor_frequency_hz <mf>     Set the motor frequency.
        update_frequency_hz <uf>    Set the update frequency.

    CONTROL_COMMAND:
        reboot                      Reboot the device.
        reset_rotation              Reset the rotation to zero at the current position.
        write_config                Write the currently set configuration to the device to keep it
    "#
    );

    Ok(())
}

fn read_state() -> Result<(), Error> {
    let hid = HidApi::new().or(Err(Error::UsbHidError))?;
    let device = hid.open(USB_VID, USB_PID).or(Err(Error::DeviceError))?;
    let mut buf = [0; std::mem::size_of::<RacingWheelState>()];

    loop {
        device
            .read(&mut buf)
            .or(Err(Error::ReadError))?;

        let mut racing_wheel_state = RacingWheelState {
            buttons: [false; 8],
            steering: f32_from_2_bytes(&buf[2..4]).unwrap(),
            throttle: f32_from_2_bytes(&buf[4..6]).unwrap(),
            ffb: f32_from_2_bytes(&buf[6..8]).unwrap(),
        };
        racing_wheel_state.buttons[0] = (buf[1] & 1) != 0;
        racing_wheel_state.buttons[1] = (buf[1] & 2) != 0;

        println!("{} {} {} {}", -racing_wheel_state.steering, racing_wheel_state.buttons[0], racing_wheel_state.buttons[1], racing_wheel_state.ffb);
    }
}


pub const LOGICAL_MAXIMUM: i32 = 10_000;
fn f32_from_2_bytes(bytes: &[u8]) -> Option<f32> {
    Some(i16::from_le_bytes([*bytes.get(0)?, *bytes.get(1)?]) as f32 / LOGICAL_MAXIMUM as f32)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let res = match args.get(1).unwrap_or(&String::new()).as_str() {
        "config" => set_option(args[2..].iter()),
        "control" => send_control_command(args[2..].iter()),
        "read_config" => read_config_action(),
        "read_state" => read_state(),
        "help" => print_help(),
        "" => Err(Error::NotEnoughArguments),
        _ => Err(Error::InvalidArgument),
    };

    match res {
        Ok(()) => println!("Success"),
        Err(Error::UsbHidError) => eprintln!("Error: Usb error"),
        Err(Error::DeviceError) => eprintln!("Error: Could not open usb device"),
        Err(Error::SendError) => eprintln!("Error: Could not send to device"),
        Err(Error::ReadError) => eprintln!("Error: Could not read from device"),
        Err(Error::InvalidArgument) => eprintln!("Error: Invalid argument, try `configurator help` to see help page"),
        Err(Error::NotEnoughArguments) => eprintln!("Error: Not enough arguments, try `configurator help` to see help page"),
        Err(Error::ParseError) => eprintln!("Error: Parse error, try `configurator help` to see help page"),
    }
}
