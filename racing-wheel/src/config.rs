use core::convert::TryFrom;

use stm32f1xx_hal::flash::{FlashWriter, FLASH_START};

const CONFIG_PAGE_PADDING: usize = 1024 - ::core::mem::size_of::<Config>();

// Page in flash memory where the config is stored
const CONFIG_PAGE: ConfigPage = ConfigPage {
    config: Config {
        gain: 0.1,
        max_rotation: 900,
        motor_max: 0.1,
        motor_deadband: 0.001,
    },
    _padding: [0; CONFIG_PAGE_PADDING],
};

#[repr(C, align(1024))]
struct ConfigPage {
    config: Config,
    _padding: [u8; CONFIG_PAGE_PADDING],
}

// Configuration settings
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Config {
    pub gain: f32,
    pub max_rotation: u16,
    pub motor_max: f32,
    pub motor_deadband: f32,
}

impl Config {
    pub fn read_from_memory() -> Config {
        CONFIG_PAGE.config
    }

    pub fn write_to_memory(&self, flash_writer: &mut FlashWriter) {
        let config_bytes = unsafe {
            ::core::slice::from_raw_parts(
                (self as *const Config) as *const u8,
                ::core::mem::size_of::<Config>(),
            )
        };

        let address = (&CONFIG_PAGE as *const ConfigPage) as u32 - FLASH_START;
        let _ = flash_writer.page_erase(address);
        let _ = flash_writer.write(address, config_bytes);
    }
}

// Device Control
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
