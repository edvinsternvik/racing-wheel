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

#[derive(Debug)]
struct Configuration {
    gain: f32,
    expo: f32,
    max_rotation: u16,
    spring_gain: f32,
    spring_coefficient: f32,
    spring_saturation: f32,
    spring_deadband: f32,
    motor_max: f32,
    motor_deadband: f32,
    motor_frequency_hz: u16,
    update_frequency_hz: u16,
}

#[derive(Clone, Copy, Debug)]
pub enum WheelDeviceControl {
    Reboot = 0x01,
    ResetRotation = 0x02,
    WriteConfig = 0x03,
}

fn send_config(device: &HidDevice, config: Configuration) -> Result<(), Error> {
    let buf = [
        CONFIG_REPORT_ID,
        f32::to_le_bytes(config.gain)[0],
        f32::to_le_bytes(config.gain)[1],
        f32::to_le_bytes(config.gain)[2],
        f32::to_le_bytes(config.gain)[3],
        f32::to_le_bytes(config.expo)[0],
        f32::to_le_bytes(config.expo)[1],
        f32::to_le_bytes(config.expo)[2],
        f32::to_le_bytes(config.expo)[3],
        u16::to_le_bytes(config.max_rotation)[0],
        u16::to_le_bytes(config.max_rotation)[1],
        f32::to_le_bytes(config.spring_gain)[0],
        f32::to_le_bytes(config.spring_gain)[1],
        f32::to_le_bytes(config.spring_gain)[2],
        f32::to_le_bytes(config.spring_gain)[3],
        f32::to_le_bytes(config.spring_coefficient)[0],
        f32::to_le_bytes(config.spring_coefficient)[1],
        f32::to_le_bytes(config.spring_coefficient)[2],
        f32::to_le_bytes(config.spring_coefficient)[3],
        f32::to_le_bytes(config.spring_saturation)[0],
        f32::to_le_bytes(config.spring_saturation)[1],
        f32::to_le_bytes(config.spring_saturation)[2],
        f32::to_le_bytes(config.spring_saturation)[3],
        f32::to_le_bytes(config.spring_deadband)[0],
        f32::to_le_bytes(config.spring_deadband)[1],
        f32::to_le_bytes(config.spring_deadband)[2],
        f32::to_le_bytes(config.spring_deadband)[3],
        f32::to_le_bytes(config.motor_max)[0],
        f32::to_le_bytes(config.motor_max)[1],
        f32::to_le_bytes(config.motor_max)[2],
        f32::to_le_bytes(config.motor_max)[3],
        f32::to_le_bytes(config.motor_deadband)[0],
        f32::to_le_bytes(config.motor_deadband)[1],
        f32::to_le_bytes(config.motor_deadband)[2],
        f32::to_le_bytes(config.motor_deadband)[3],
        u16::to_le_bytes(config.motor_frequency_hz)[0],
        u16::to_le_bytes(config.motor_frequency_hz)[1],
        u16::to_le_bytes(config.update_frequency_hz)[0],
        u16::to_le_bytes(config.update_frequency_hz)[1],
    ];
    device.send_feature_report(&buf).or(Err(Error::SendError))
}

fn read_config(device: &HidDevice) -> Result<Configuration, Error> {
    let mut buf = [0; 39];
    buf[0] = CONFIG_REPORT_ID;

    let bytes_read = device
        .get_feature_report(&mut buf)
        .or(Err(Error::ReadError))?;

    if bytes_read != buf.len() {
        return Err(Error::ReadError);
    }

    Ok(Configuration {
        gain: f32::from_le_bytes([buf[1], buf[2], buf[3], buf[4]]),
        expo: f32::from_le_bytes([buf[5], buf[6], buf[7], buf[8]]),
        max_rotation: u16::from_le_bytes([buf[9], buf[10]]),
        spring_gain: f32::from_le_bytes([buf[11], buf[12], buf[13], buf[14]]),
        spring_coefficient: f32::from_le_bytes([buf[15], buf[16], buf[17], buf[18]]),
        spring_saturation: f32::from_le_bytes([buf[19], buf[20], buf[21], buf[22]]),
        spring_deadband: f32::from_le_bytes([buf[23], buf[24], buf[25], buf[26]]),
        motor_max: f32::from_le_bytes([buf[27], buf[28], buf[29], buf[30]]),
        motor_deadband: f32::from_le_bytes([buf[31], buf[32], buf[33], buf[34]]),
        motor_frequency_hz: u16::from_le_bytes([buf[35], buf[36]]),
        update_frequency_hz: u16::from_le_bytes([buf[37], buf[38]]),
    })
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
