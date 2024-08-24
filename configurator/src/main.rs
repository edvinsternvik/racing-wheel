use config::{config::Config, control::WheelDeviceControl};
use hidapi::{HidApi, HidDevice};
use std::slice::Iter;

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
    let mut buf = [0; 39];
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
        "max_rotation" => config.max_rotation = value.parse().or(Err(Error::ParseError))?,
        "spring_gain" => config.spring_gain = value.parse().or(Err(Error::ParseError))?,
        "spring_coefficient" => {
            config.spring_coefficient = value.parse().or(Err(Error::ParseError))?
        }
        "spring_saturation" => {
            config.spring_saturation = value.parse().or(Err(Error::ParseError))?
        }
        "spring_deadband" => config.spring_deadband = value.parse().or(Err(Error::ParseError))?,
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

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let res = match args.get(1).unwrap_or(&String::new()).as_str() {
        "config" => set_option(args[2..].iter()),
        "control" => send_control_command(args[2..].iter()),
        "read_config" => read_config_action(),
        "" => Err(Error::NotEnoughArguments),
        _ => Err(Error::InvalidArgument),
    };

    match res {
        Ok(()) => println!("Success"),
        Err(Error::UsbHidError) => eprintln!("Error: Usb error"),
        Err(Error::DeviceError) => eprintln!("Error: Could not open usb device"),
        Err(Error::SendError) => eprintln!("Error: Could not send to device"),
        Err(Error::ReadError) => eprintln!("Error: Could not read from device"),
        Err(Error::InvalidArgument) => eprintln!("Error: Invalid argument"),
        Err(Error::NotEnoughArguments) => eprintln!("Error: Not enough arguments"),
        Err(Error::ParseError) => eprintln!("Error: Parse error"),
    }
}
