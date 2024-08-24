#[derive(Clone, Copy, Debug)]
pub enum WheelDeviceControl {
    Reboot = 0x01,
    ResetRotation = 0x02,
    WriteConfig = 0x03,
}

impl TryFrom<u8> for WheelDeviceControl {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        use WheelDeviceControl::*;
        match value {
            v if v == Reboot as u8 => Ok(Reboot),
            v if v == ResetRotation as u8 => Ok(ResetRotation),
            v if v == WriteConfig as u8 => Ok(WriteConfig),
            _ => Err(()),
        }
    }
}
