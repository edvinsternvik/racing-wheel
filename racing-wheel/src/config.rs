use config::config::Config;
use stm32f1xx_hal::flash::{FlashWriter, FLASH_START};

const CONFIG_PAGE_PADDING: usize = 1024 - ::core::mem::size_of::<Config>();

// Page in flash memory where the config is stored
const CONFIG_PAGE: ConfigPage = ConfigPage {
    config: Config {
        gain: 0.3,
        expo: 0.8,
        max_rotation: 360,
        spring_gain: 0.45,
        spring_coefficient: 8.0,
        spring_saturation: 1.0,
        spring_deadband: 0.0001,
        damper_gain: 0.8,
        damper_coefficient: 0.4,
        damper_saturation: 1.0,
        damper_deadband: 0.0001,
        motor_max: 0.8,
        motor_deadband: 0.0001,
        motor_frequency_hz: 20_000,
        update_frequency_hz: 100,
    },
    _padding: [0; CONFIG_PAGE_PADDING],
};

#[repr(C, align(1024))]
struct ConfigPage {
    config: Config,
    _padding: [u8; CONFIG_PAGE_PADDING],
}

pub trait FlashMemoryData {
    fn read_from_memory(flash_writer: &FlashWriter) -> Config;
    fn write_to_memory(&self, flash_writer: &mut FlashWriter);
}

impl FlashMemoryData for Config {
    fn read_from_memory(flash_writer: &FlashWriter) -> Config {
        let address = (&CONFIG_PAGE.config as *const Config) as u32 - FLASH_START;
        let conf_bytes = flash_writer.read(address, size_of::<Config>()).unwrap();
        let conf = unsafe { *(conf_bytes.as_ptr() as *const Config) };

        conf
    }

    fn write_to_memory(&self, flash_writer: &mut FlashWriter) {
        let config_bytes = unsafe {
            ::core::slice::from_raw_parts(
                (self as *const Config) as *const u8,
                ::core::mem::size_of::<Config>(),
            )
        };

        let address = (&CONFIG_PAGE.config as *const Config) as u32 - FLASH_START;
        let _ = flash_writer.page_erase(address);
        let _ = flash_writer.write(address, config_bytes);
    }
}
