use crate::hid::{HIDReportIn, HIDReportOut, ReportID, ReportType};

// Create New Effect Report
pub struct CreateNewEffectReport {
    pub effect_type: u8,
    pub byte_count: u16,
}

impl HIDReportIn<4> for CreateNewEffectReport {
    const ID: ReportID = ReportID(ReportType::Feature, 0x01);
    fn report_bytes(&self) -> [u8; 4] {
        [
            Self::ID.1,
            self.effect_type,
            self.byte_count.to_le_bytes()[0],
            self.byte_count.to_le_bytes()[1],
        ]
    }
}

// PID Block Load Report
#[derive(Clone, Copy)]
pub enum BlockLoadStatus {
    Success = 0x01,
    _Full = 0x02,
    _Error = 0x03,
}

pub struct PIDBlockLoadReport {
    pub effect_block_index: u8,
    pub block_load_status: BlockLoadStatus,
    pub ram_pool_available: u16,
}

impl HIDReportIn<5> for PIDBlockLoadReport {
    const ID: ReportID = ReportID(ReportType::Feature, 0x02);
    fn report_bytes(&self) -> [u8; 5] {
        [
            Self::ID.1,
            self.effect_block_index,
            self.block_load_status as u8,
            self.ram_pool_available.to_le_bytes()[0],
            self.ram_pool_available.to_le_bytes()[1],
        ]
    }
}

// PID Pool Report
pub struct PIDPoolReport {
    pub ram_pool_size: u16,
    pub simultaneous_effects_max: u8,
    pub device_managed_pool: bool,
    pub shared_parameter_blocks: bool,
}

impl HIDReportIn<5> for PIDPoolReport {
    const ID: ReportID = ReportID(ReportType::Feature, 0x03);
    fn report_bytes(&self) -> [u8; 5] {
        [
            Self::ID.1,
            self.ram_pool_size.to_le_bytes()[0],
            self.ram_pool_size.to_le_bytes()[1],
            self.simultaneous_effects_max,
            bitflags(&[self.device_managed_pool, self.shared_parameter_blocks]),
        ]
    }
}

pub struct PIDBlockFreeReport {
    pub effect_block_index: u8,
}

impl HIDReportOut for PIDBlockFreeReport {
    const ID: ReportID = ReportID(ReportType::Output, 0x0B);
    fn into_report(bytes: &[u8]) -> Self {
        Self {
            effect_block_index: bytes[0],
        }
    }
}

pub struct JoystickReport {
    pub buttons: [bool; 8],
    pub joystick_x: u16,
    pub joystick_y: u16,
}

impl HIDReportIn<6> for JoystickReport {
    const ID: ReportID = ReportID(ReportType::Input, 0x01);
    fn report_bytes(&self) -> [u8; 6] {
        [
            Self::ID.1,
            bitflags(&self.buttons),
            self.joystick_x.to_le_bytes()[0],
            self.joystick_x.to_le_bytes()[1],
            self.joystick_y.to_le_bytes()[0],
            self.joystick_y.to_le_bytes()[1],
        ]
    }
}

pub struct PIDStateReport {
    pub device_paused: bool,
    pub actuators_enabled: bool,
    pub safety_switch: bool,
    pub actuators_override_switch: bool,
    pub actuator_power: bool,
    pub effect_playing: bool,
    pub effect_block_index: u8,
}

impl HIDReportIn<3> for PIDStateReport {
    const ID: ReportID = ReportID(ReportType::Input, 0x02);
    fn report_bytes(&self) -> [u8; 3] {
        [
            Self::ID.1,
            bitflags(&[
                self.device_paused,
                self.actuators_enabled,
                self.safety_switch,
                self.actuators_override_switch,
                self.actuator_power,
            ]),
            bitflags(&[self.effect_playing]) | (self.effect_block_index << 1),
        ]
    }
}

fn bitflags(flags: &[bool]) -> u8 {
    flags
        .into_iter()
        .enumerate()
        .fold(0, |b, (i, flag)| b | (*flag as u8) << i)
}
