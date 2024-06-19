use core::convert::{TryFrom, TryInto};

use crate::hid::{HIDReportIn, HIDReportOut, ReportID, ReportType};

// Joystick report
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

// PID State Report
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

// Set Effect Report
pub struct SetEffectReport {
    effect_block_index: u8,
    effect_type: Option<EffectType>,
    duration: u16,
    trigger_repeat_interval: u16,
    sample_period: u16,
    gain: u8,
    trigger_button: u8,
    axis_x_enable: bool,
    axis_y_enable: bool,
    direction_enable: bool,
    direction_instance_1: u8,
    direction_instance_2: u8,
    start_delay: u16,
    type_specific_block_offset_instance_1: u16,
    type_specific_block_offset_instance_2: u16,
}

impl HIDReportOut for SetEffectReport {
    const ID: ReportID = ReportID(ReportType::Output, 0x01);
    fn into_report(bytes: &[u8]) -> Self {
        Self {
            effect_block_index: bytes[0],
            effect_type: EffectType::try_from(bytes[1]).ok(),
            duration: u16::from_le_bytes([bytes[2], bytes[3]]),
            trigger_repeat_interval: u16::from_le_bytes([bytes[4], bytes[5]]),
            sample_period: u16::from_le_bytes([bytes[6], bytes[7]]),
            gain: bytes[8],
            trigger_button: bytes[9],
            axis_x_enable: bitflag(bytes[10], 0),
            axis_y_enable: bitflag(bytes[10], 1),
            direction_enable: bitflag(bytes[10], 2),
            direction_instance_1: bytes[11],
            direction_instance_2: bytes[12],
            start_delay: u16::from_le_bytes([bytes[13], bytes[14]]),
            type_specific_block_offset_instance_1: u16::from_le_bytes([bytes[15], bytes[16]]),
            type_specific_block_offset_instance_2: u16::from_le_bytes([bytes[17], bytes[18]]),
        }
    }
}

#[derive(Clone, Copy)]
pub enum EffectType {
    ConstantForce = 1,
    Ramp = 2,
    Square = 3,
    Sine = 4,
    Triangle = 5,
    SawtoothUp = 6,
    SawtoothDown = 7,
    Spring = 8,
    Damper = 9,
    Inertia = 10,
    Friction = 11,
    CustomForceData = 12,
}

impl TryFrom<u8> for EffectType {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        use EffectType::*;
        match value {
            x if x == ConstantForce as u8 => Ok(ConstantForce),
            x if x == Ramp as u8 => Ok(Ramp),
            x if x == Square as u8 => Ok(Square),
            x if x == Sine as u8 => Ok(Sine),
            x if x == Triangle as u8 => Ok(Triangle),
            x if x == SawtoothUp as u8 => Ok(SawtoothUp),
            x if x == SawtoothDown as u8 => Ok(SawtoothDown),
            x if x == Spring as u8 => Ok(Spring),
            x if x == Damper as u8 => Ok(Damper),
            x if x == Inertia as u8 => Ok(Inertia),
            x if x == Friction as u8 => Ok(Friction),
            x if x == CustomForceData as u8 => Ok(CustomForceData),
            _ => Err(()),
        }
    }
}

// Set Envelope Report
pub struct SetEnvelopeReport {
    effect_block_index: u8,
    attack_level: u16,
    fade_level: u16,
    attack_time: u32,
    fade_time: u32,
}

impl HIDReportOut for SetEnvelopeReport {
    const ID: ReportID = ReportID(ReportType::Output, 0x02);
    fn into_report(bytes: &[u8]) -> Self {
        Self {
            effect_block_index: bytes[0],
            attack_level: u16::from_le_bytes([bytes[1], bytes[2]]),
            fade_level: u16::from_le_bytes([bytes[3], bytes[4]]),
            attack_time: u32::from_le_bytes([bytes[5], bytes[6], bytes[7], bytes[8]]),
            fade_time: u32::from_le_bytes([bytes[9], bytes[10], bytes[11], bytes[12]]),
        }
    }
}

// Set Condition Report
pub struct SetConditionReport {
    effect_block_index: u8,
    parameter_block_offset: u8,
    type_specific_block_offset_instance_1: u8,
    type_specific_block_offset_instance_2: u8,
    cp_offset: u16,
    positive_coefficient: u16,
    negative_coefficient: u16,
    positive_saturation: u16,
    negative_saturation: u16,
    dead_band: u16,
}

impl HIDReportOut for SetConditionReport {
    const ID: ReportID = ReportID(ReportType::Output, 0x03);
    fn into_report(bytes: &[u8]) -> Self {
        Self {
            effect_block_index: bytes[0],
            parameter_block_offset: bits(bytes[1], 0, 4),
            type_specific_block_offset_instance_1: bits(bytes[1], 4, 2),
            type_specific_block_offset_instance_2: bits(bytes[1], 6, 2),
            cp_offset: u16::from_le_bytes([bytes[2], bytes[3]]),
            positive_coefficient: u16::from_le_bytes([bytes[4], bytes[5]]),
            negative_coefficient: u16::from_le_bytes([bytes[6], bytes[7]]),
            positive_saturation: u16::from_le_bytes([bytes[8], bytes[9]]),
            negative_saturation: u16::from_le_bytes([bytes[10], bytes[11]]),
            dead_band: u16::from_le_bytes([bytes[12], bytes[13]]),
        }
    }
}

// Set Periodic Report
pub struct SetPeriodicReport {
    effect_block_index: u8,
    magnitude: u16,
    offset: u16,
    phase: u16,
    period: u32,
}

impl HIDReportOut for SetPeriodicReport {
    const ID: ReportID = ReportID(ReportType::Output, 0x04);
    fn into_report(bytes: &[u8]) -> Self {
        Self {
            effect_block_index: bytes[0],
            magnitude: u16::from_le_bytes([bytes[1], bytes[2]]),
            offset: u16::from_le_bytes([bytes[3], bytes[4]]),
            phase: u16::from_le_bytes([bytes[5], bytes[6]]),
            period: u32::from_le_bytes([bytes[7], bytes[8], bytes[9], bytes[10]]),
        }
    }
}

// Set Constant Force Report
pub struct SetConstantForceReport {
    effect_block_index: u8,
    magnitude: u16,
}

impl HIDReportOut for SetConstantForceReport {
    const ID: ReportID = ReportID(ReportType::Output, 0x05);
    fn into_report(bytes: &[u8]) -> Self {
        Self {
            effect_block_index: bytes[0],
            magnitude: u16::from_le_bytes([bytes[1], bytes[2]]),
        }
    }
}

// Set Ramp Force Report
pub struct SetRampForceReport {
    effect_block_index: u8,
    ramp_start: u16,
    ramp_end: u16,
}

impl HIDReportOut for SetRampForceReport {
    const ID: ReportID = ReportID(ReportType::Output, 0x06);
    fn into_report(bytes: &[u8]) -> Self {
        Self {
            effect_block_index: bytes[0],
            ramp_start: u16::from_le_bytes([bytes[1], bytes[2]]),
            ramp_end: u16::from_le_bytes([bytes[3], bytes[4]]),
        }
    }
}

// Custom Force Data Report
pub struct CustomForceDataReport {
    effect_block_index: u8,
    custom_force_data_offset: u16,
    custom_force_data: [u8; 12],
}

impl HIDReportOut for CustomForceDataReport {
    const ID: ReportID = ReportID(ReportType::Output, 0x07);
    fn into_report(bytes: &[u8]) -> Self {
        Self {
            effect_block_index: bytes[0],
            custom_force_data_offset: u16::from_le_bytes([bytes[1], bytes[2]]),
            custom_force_data: bytes[3..(3+12)].try_into().unwrap_or_default(),
        }
    }
}

// Download Force Sample
pub struct DownloadForceSample {
    axis_x: u8,
    axis_y: u8,
}

impl HIDReportOut for DownloadForceSample {
    const ID: ReportID = ReportID(ReportType::Output, 0x08);
    fn into_report(bytes: &[u8]) -> Self {
        Self {
            axis_x: bytes[0],
            axis_y: bytes[1],
        }
    }
}

// Effect Operation Report
pub struct EffectOperationReport {
    effect_block_index: u8,
    effect_operation: Option<EffectOperation>,
    loop_count: u8,

}

impl HIDReportOut for EffectOperationReport {
    const ID: ReportID = ReportID(ReportType::Output, 0x0A);
    fn into_report(bytes: &[u8]) -> Self {
        Self {
            effect_block_index: bytes[0],
            effect_operation: EffectOperation::try_from(bytes[1]).ok(),
            loop_count: bytes[2],
        }
    }
}

pub enum EffectOperation {
    EffectStart = 1,
    EffectStartSolo = 2,
    EffectStop = 3,
}

impl TryFrom<u8> for EffectOperation {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        use EffectOperation::*;
        match value {
            x if x == EffectStart as u8 => Ok(EffectStart),
            x if x == EffectStartSolo as u8 => Ok(EffectStartSolo),
            x if x == EffectStop as u8 => Ok(EffectStop),
            _ => Err(()),
        }
    }
}

// PID Block Free Report
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

// PID Device Control
pub struct PIDDeviceControl {
    device_control: Option<DeviceControl>,
}

impl HIDReportOut for PIDDeviceControl {
    const ID: ReportID = ReportID(ReportType::Output, 0x0C);
    fn into_report(bytes: &[u8]) -> Self {
        Self {
            device_control: DeviceControl::try_from(bytes[0]).ok(),
        }
    }
}

pub enum DeviceControl {
    EnableActuators = 1,
    DisableActuators = 2,
    StopAllEffects = 3,
    DeviceReset = 4,
    DevicePause = 5,
    DeviceContinue = 6,
}

impl TryFrom<u8> for DeviceControl {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        use DeviceControl::*;
        match value {
            x if x == EnableActuators as u8 => Ok(EnableActuators),
            x if x == DisableActuators as u8 => Ok(DisableActuators),
            x if x == StopAllEffects as u8 => Ok(StopAllEffects),
            x if x == DeviceReset as u8 => Ok(DeviceReset),
            x if x == DevicePause as u8 => Ok(DevicePause),
            x if x == DeviceContinue as u8 => Ok(DeviceContinue),
            _ => Err(()),
        }
    }
}

// Device Gain Report
pub struct DeviceGainReport {
    device_gain: u8,
}

impl HIDReportOut for DeviceGainReport {
    const ID: ReportID = ReportID(ReportType::Output, 0x0D);
    fn into_report(bytes: &[u8]) -> Self {
        Self {
            device_gain: bytes[0],
        }
    }
}

// Set Custom Force Report
pub struct SetCustomForceReport {
    effect_block_index: u8,
    sample_count: u8,
    sample_period: u16,
}

impl HIDReportOut for SetCustomForceReport {
    const ID: ReportID = ReportID(ReportType::Output, 0x0E);
    fn into_report(bytes: &[u8]) -> Self {
        Self {
            effect_block_index: bytes[0],
            sample_count: bytes[1],
            sample_period: u16::from_le_bytes([bytes[2], bytes[3]]),
        }
    }
}

// Create New Effect Report
pub struct CreateNewEffectReport {
    pub effect_type: EffectType,
    pub byte_count: u16,
}

impl HIDReportIn<4> for CreateNewEffectReport {
    const ID: ReportID = ReportID(ReportType::Feature, 0x01);
    fn report_bytes(&self) -> [u8; 4] {
        [
            Self::ID.1,
            self.effect_type as u8,
            self.byte_count.to_le_bytes()[0],
            self.byte_count.to_le_bytes()[1],
        ]
    }
}

// PID Block Load Report
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

#[derive(Clone, Copy)]
pub enum BlockLoadStatus {
    Success = 0x01,
    _Full = 0x02,
    _Error = 0x03,
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


fn bitflags(flags: &[bool]) -> u8 {
    flags
        .into_iter()
        .enumerate()
        .fold(0, |b, (i, flag)| b | (*flag as u8) << i)
}

fn bitflag(flags: u8, i: u8) -> bool {
    (flags & (1 << i)) != 0
}

fn bits(byte: u8, start: u8, n_bits: u8) -> u8 {
    (byte << i32::max(0_i32, 8_i32 - start as i32 - n_bits as i32)) >> start
}
