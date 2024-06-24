use crate::{
    hid_device::{HIDReport, HIDReportIn, HIDReportOut, HIDReportRAM, ReportID, ReportType},
    misc::{bitflag, bitflags, bits},
};
use core::convert::{TryFrom, TryInto};

// Racing wheel report
#[derive(Default, Clone)]
pub struct RacingWheelReport {
    pub buttons: [bool; 8],
    pub steering: i16,
    pub throttle: i16,
}

impl HIDReport for RacingWheelReport {
    const ID: ReportID = ReportID(ReportType::Input, 0x01);
}

impl HIDReportIn<6> for RacingWheelReport {
    fn report_bytes(&self) -> [u8; 6] {
        [
            Self::ID.1,
            bitflags(&self.buttons),
            self.steering.to_le_bytes()[0],
            self.steering.to_le_bytes()[1],
            self.throttle.to_le_bytes()[0],
            self.throttle.to_le_bytes()[1],
        ]
    }
}

// PID State Report
#[derive(Default, Clone)]
pub struct PIDStateReport {
    pub device_paused: bool,
    pub actuators_enabled: bool,
    pub safety_switch: bool,
    pub actuators_override_switch: bool,
    pub actuator_power: bool,
    pub effect_playing: bool,
    pub effect_block_index: u8,
}

impl HIDReport for PIDStateReport {
    const ID: ReportID = ReportID(ReportType::Input, 0x02);
}

impl HIDReportIn<3> for PIDStateReport {
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
    pub effect_block_index: u8,
    pub effect_type: EffectType,
    pub duration: u16,
    pub trigger_repeat_interval: u16,
    pub sample_period: u16,
    pub gain: u8,
    pub trigger_button: u8,
    pub axis_x_enable: bool,
    pub axis_y_enable: bool,
    pub direction_enable: bool,
    pub direction_instance_1: u8,
    pub direction_instance_2: u8,
    pub start_delay: u16,
    pub type_specific_block_offset_instance_1: u16,
    pub type_specific_block_offset_instance_2: u16,
}

impl HIDReport for SetEffectReport {
    const ID: ReportID = ReportID(ReportType::Output, 0x01);
}

impl HIDReportOut for SetEffectReport {
    fn into_report(bytes: &[u8]) -> Option<Self> {
        Self::from_ram(&bytes[2..], *bytes.get(1)?)
    }
}

impl HIDReportRAM<18> for SetEffectReport {
    fn from_ram(ram: &[u8], effect_block_index: u8) -> Option<Self> {
        Some(Self {
            effect_block_index,
            effect_type: EffectType::try_from(*ram.get(0)?).ok()?,
            duration: u16::from_le_bytes([*ram.get(1)?, *ram.get(2)?]),
            trigger_repeat_interval: u16::from_le_bytes([*ram.get(3)?, *ram.get(4)?]),
            sample_period: u16::from_le_bytes([*ram.get(5)?, *ram.get(6)?]),
            gain: *ram.get(7)?,
            trigger_button: *ram.get(8)?,
            axis_x_enable: bitflag(*ram.get(9)?, 0),
            axis_y_enable: bitflag(*ram.get(9)?, 1),
            direction_enable: bitflag(*ram.get(9)?, 2),
            direction_instance_1: *ram.get(10)?,
            direction_instance_2: *ram.get(11)?,
            start_delay: u16::from_le_bytes([*ram.get(12)?, *ram.get(13)?]),
            type_specific_block_offset_instance_1: u16::from_le_bytes([
                *ram.get(14)?,
                *ram.get(15)?,
            ]),
            type_specific_block_offset_instance_2: u16::from_le_bytes([
                *ram.get(16)?,
                *ram.get(17)?,
            ]),
        })
    }

    fn to_ram(&self) -> [u8; 18] {
        [
            self.effect_type as u8,
            self.duration.to_le_bytes()[0],
            self.duration.to_le_bytes()[1],
            self.trigger_repeat_interval.to_le_bytes()[0],
            self.trigger_repeat_interval.to_le_bytes()[1],
            self.sample_period.to_le_bytes()[0],
            self.sample_period.to_le_bytes()[1],
            self.gain,
            self.trigger_button,
            bitflags(&[
                self.axis_x_enable,
                self.axis_y_enable,
                self.direction_enable,
            ]),
            self.direction_instance_1,
            self.direction_instance_2,
            self.start_delay.to_le_bytes()[0],
            self.start_delay.to_le_bytes()[1],
            self.type_specific_block_offset_instance_1.to_le_bytes()[0],
            self.type_specific_block_offset_instance_1.to_le_bytes()[1],
            self.type_specific_block_offset_instance_2.to_le_bytes()[0],
            self.type_specific_block_offset_instance_2.to_le_bytes()[1],
        ]
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
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
    pub effect_block_index: u8,
    pub attack_level: u16,
    pub fade_level: u16,
    pub attack_time: u32,
    pub fade_time: u32,
}

impl HIDReport for SetEnvelopeReport {
    const ID: ReportID = ReportID(ReportType::Output, 0x02);
}

impl HIDReportOut for SetEnvelopeReport {
    fn into_report(bytes: &[u8]) -> Option<Self> {
        Self::from_ram(&bytes[2..], *bytes.get(1)?)
    }
}

impl HIDReportRAM<12> for SetEnvelopeReport {
    fn from_ram(ram: &[u8], effect_block_index: u8) -> Option<Self> {
        Some(Self {
            effect_block_index,
            attack_level: u16::from_le_bytes([*ram.get(0)?, *ram.get(1)?]),
            fade_level: u16::from_le_bytes([*ram.get(2)?, *ram.get(3)?]),
            attack_time: u32::from_le_bytes([
                *ram.get(4)?,
                *ram.get(5)?,
                *ram.get(6)?,
                *ram.get(7)?,
            ]),
            fade_time: u32::from_le_bytes([
                *ram.get(8)?,
                *ram.get(9)?,
                *ram.get(10)?,
                *ram.get(11)?,
            ]),
        })
    }

    fn to_ram(&self) -> [u8; 12] {
        [
            self.attack_level.to_le_bytes()[0],
            self.attack_level.to_le_bytes()[1],
            self.fade_level.to_le_bytes()[0],
            self.fade_level.to_le_bytes()[1],
            self.attack_time.to_le_bytes()[0],
            self.attack_time.to_le_bytes()[1],
            self.attack_time.to_le_bytes()[2],
            self.attack_time.to_le_bytes()[3],
            self.fade_time.to_le_bytes()[0],
            self.fade_time.to_le_bytes()[1],
            self.fade_time.to_le_bytes()[2],
            self.fade_time.to_le_bytes()[3],
        ]
    }
}

// Set Condition Report
pub struct SetConditionReport {
    pub effect_block_index: u8,
    pub parameter_block_offset: u8,
    pub type_specific_block_offset_instance_1: u8,
    pub type_specific_block_offset_instance_2: u8,
    pub cp_offset: u16,
    pub positive_coefficient: u16,
    pub negative_coefficient: u16,
    pub positive_saturation: u16,
    pub negative_saturation: u16,
    pub dead_band: u16,
}

impl HIDReport for SetConditionReport {
    const ID: ReportID = ReportID(ReportType::Output, 0x03);
}

impl HIDReportOut for SetConditionReport {
    fn into_report(bytes: &[u8]) -> Option<Self> {
        Self::from_ram(&bytes[2..], *bytes.get(1)?)
    }
}

impl HIDReportRAM<13> for SetConditionReport {
    fn from_ram(ram: &[u8], effect_block_index: u8) -> Option<Self> {
        Some(Self {
            effect_block_index,
            parameter_block_offset: bits(*ram.get(0)?, 0, 4),
            type_specific_block_offset_instance_1: bits(*ram.get(0)?, 4, 2),
            type_specific_block_offset_instance_2: bits(*ram.get(0)?, 6, 2),
            cp_offset: u16::from_le_bytes([*ram.get(1)?, *ram.get(2)?]),
            positive_coefficient: u16::from_le_bytes([*ram.get(3)?, *ram.get(4)?]),
            negative_coefficient: u16::from_le_bytes([*ram.get(5)?, *ram.get(6)?]),
            positive_saturation: u16::from_le_bytes([*ram.get(7)?, *ram.get(8)?]),
            negative_saturation: u16::from_le_bytes([*ram.get(9)?, *ram.get(10)?]),
            dead_band: u16::from_le_bytes([*ram.get(11)?, *ram.get(12)?]),
        })
    }

    fn to_ram(&self) -> [u8; 13] {
        [
            (self.parameter_block_offset & 0b1111) << 0
                | (self.type_specific_block_offset_instance_1 & 0b11) << 4
                | (self.type_specific_block_offset_instance_2 & 0b11) << 6,
            self.cp_offset.to_le_bytes()[0],
            self.cp_offset.to_le_bytes()[1],
            self.positive_coefficient.to_le_bytes()[0],
            self.positive_coefficient.to_le_bytes()[1],
            self.negative_coefficient.to_le_bytes()[0],
            self.negative_coefficient.to_le_bytes()[1],
            self.positive_saturation.to_le_bytes()[0],
            self.positive_saturation.to_le_bytes()[1],
            self.negative_saturation.to_le_bytes()[0],
            self.negative_saturation.to_le_bytes()[1],
            self.dead_band.to_le_bytes()[0],
            self.dead_band.to_le_bytes()[1],
        ]
    }
}

// Set Periodic Report
pub struct SetPeriodicReport {
    pub effect_block_index: u8,
    pub magnitude: u16,
    pub offset: u16,
    pub phase: u16,
    pub period: u32,
}

impl HIDReport for SetPeriodicReport {
    const ID: ReportID = ReportID(ReportType::Output, 0x04);
}

impl HIDReportOut for SetPeriodicReport {
    fn into_report(bytes: &[u8]) -> Option<Self> {
        Self::from_ram(&bytes[2..], *bytes.get(1)?)
    }
}

impl HIDReportRAM<10> for SetPeriodicReport {
    fn from_ram(ram: &[u8], effect_block_index: u8) -> Option<Self> {
        Some(Self {
            effect_block_index,
            magnitude: u16::from_le_bytes([*ram.get(0)?, *ram.get(1)?]),
            offset: u16::from_le_bytes([*ram.get(2)?, *ram.get(3)?]),
            phase: u16::from_le_bytes([*ram.get(4)?, *ram.get(5)?]),
            period: u32::from_le_bytes([*ram.get(6)?, *ram.get(7)?, *ram.get(8)?, *ram.get(9)?]),
        })
    }

    fn to_ram(&self) -> [u8; 10] {
        [
            self.magnitude.to_le_bytes()[0],
            self.magnitude.to_le_bytes()[1],
            self.offset.to_le_bytes()[0],
            self.offset.to_le_bytes()[1],
            self.phase.to_le_bytes()[0],
            self.phase.to_le_bytes()[1],
            self.period.to_le_bytes()[0],
            self.period.to_le_bytes()[1],
            self.period.to_le_bytes()[2],
            self.period.to_le_bytes()[3],
        ]
    }
}

// Set Constant Force Report
pub struct SetConstantForceReport {
    pub effect_block_index: u8,
    pub magnitude: u16,
}

impl HIDReport for SetConstantForceReport {
    const ID: ReportID = ReportID(ReportType::Output, 0x05);
}

impl HIDReportOut for SetConstantForceReport {
    fn into_report(bytes: &[u8]) -> Option<Self> {
        Self::from_ram(&bytes[2..], *bytes.get(1)?)
    }
}

impl HIDReportRAM<2> for SetConstantForceReport {
    fn from_ram(ram: &[u8], effect_block_index: u8) -> Option<Self> {
        Some(Self {
            effect_block_index,
            magnitude: u16::from_le_bytes([*ram.get(0)?, *ram.get(1)?]),
        })
    }

    fn to_ram(&self) -> [u8; 2] {
        [
            self.magnitude.to_le_bytes()[0],
            self.magnitude.to_le_bytes()[1],
        ]
    }
}

// Set Ramp Force Report
pub struct SetRampForceReport {
    pub effect_block_index: u8,
    pub ramp_start: u16,
    pub ramp_end: u16,
}

impl HIDReport for SetRampForceReport {
    const ID: ReportID = ReportID(ReportType::Output, 0x06);
}

impl HIDReportOut for SetRampForceReport {
    fn into_report(bytes: &[u8]) -> Option<Self> {
        Self::from_ram(&bytes[2..], *bytes.get(1)?)
    }
}

impl HIDReportRAM<4> for SetRampForceReport {
    fn from_ram(ram: &[u8], effect_block_index: u8) -> Option<Self> {
        Some(Self {
            effect_block_index,
            ramp_start: u16::from_le_bytes([*ram.get(0)?, *ram.get(1)?]),
            ramp_end: u16::from_le_bytes([*ram.get(2)?, *ram.get(3)?]),
        })
    }

    fn to_ram(&self) -> [u8; 4] {
        [
            self.ramp_start.to_le_bytes()[0],
            self.ramp_start.to_le_bytes()[1],
            self.ramp_end.to_le_bytes()[0],
            self.ramp_end.to_le_bytes()[1],
        ]
    }
}

// Custom Force Data Report
pub struct CustomForceDataReport {
    pub effect_block_index: u8,
    pub custom_force_data_offset: u16,
    pub byte_count: u8,
    pub custom_force_data: [u8; 12],
}

impl HIDReport for CustomForceDataReport {
    const ID: ReportID = ReportID(ReportType::Output, 0x07);
}

impl HIDReportOut for CustomForceDataReport {
    fn into_report(bytes: &[u8]) -> Option<Self> {
        Self::from_ram(&bytes[2..], *bytes.get(1)?)
    }
}

impl HIDReportRAM<15> for CustomForceDataReport {
    fn from_ram(ram: &[u8], effect_block_index: u8) -> Option<Self> {
        Some(Self {
            effect_block_index,
            custom_force_data_offset: u16::from_le_bytes([*ram.get(0)?, *ram.get(1)?]),
            byte_count: *ram.get(2)?,
            custom_force_data: ram.get(3..(3 + 12))?.try_into().unwrap_or_default(),
        })
    }

    fn to_ram(&self) -> [u8; 15] {
        [
            self.custom_force_data_offset.to_le_bytes()[0],
            self.custom_force_data_offset.to_le_bytes()[1],
            self.byte_count,
            self.custom_force_data[0],
            self.custom_force_data[1],
            self.custom_force_data[2],
            self.custom_force_data[3],
            self.custom_force_data[4],
            self.custom_force_data[5],
            self.custom_force_data[6],
            self.custom_force_data[7],
            self.custom_force_data[8],
            self.custom_force_data[9],
            self.custom_force_data[10],
            self.custom_force_data[11],
        ]
    }
}

// Download Force Sample
pub struct DownloadForceSample {
    pub steering: u8,
    pub throttle: u8,
}

impl HIDReport for DownloadForceSample {
    const ID: ReportID = ReportID(ReportType::Output, 0x08);
}

impl HIDReportOut for DownloadForceSample {
    fn into_report(bytes: &[u8]) -> Option<Self> {
        Some(Self {
            steering: *bytes.get(1)?,
            throttle: *bytes.get(2)?,
        })
    }
}

// Effect Operation Report
pub struct EffectOperationReport {
    pub effect_block_index: u8,
    pub effect_operation: EffectOperation,
    pub loop_count: u8,
}

impl HIDReport for EffectOperationReport {
    const ID: ReportID = ReportID(ReportType::Output, 0x0A);
}

impl HIDReportOut for EffectOperationReport {
    fn into_report(bytes: &[u8]) -> Option<Self> {
        Some(Self {
            effect_block_index: *bytes.get(1)?,
            effect_operation: EffectOperation::try_from(*bytes.get(2)?).ok()?,
            loop_count: *bytes.get(3)?,
        })
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

impl HIDReport for PIDBlockFreeReport {
    const ID: ReportID = ReportID(ReportType::Output, 0x0B);
}

impl HIDReportOut for PIDBlockFreeReport {
    fn into_report(bytes: &[u8]) -> Option<Self> {
        Some(Self {
            effect_block_index: *bytes.get(1)?,
        })
    }
}

// PID Device Control
pub struct PIDDeviceControl {
    pub device_control: DeviceControl,
}

impl HIDReport for PIDDeviceControl {
    const ID: ReportID = ReportID(ReportType::Output, 0x0C);
}

impl HIDReportOut for PIDDeviceControl {
    fn into_report(bytes: &[u8]) -> Option<Self> {
        Some(Self {
            device_control: DeviceControl::try_from(*bytes.get(1)?).ok()?,
        })
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
    pub device_gain: u8,
}

impl HIDReport for DeviceGainReport {
    const ID: ReportID = ReportID(ReportType::Output, 0x0D);
}

impl HIDReportOut for DeviceGainReport {
    fn into_report(bytes: &[u8]) -> Option<Self> {
        Some(Self {
            device_gain: *bytes.get(1)?,
        })
    }
}

// Set Custom Force Report
pub struct SetCustomForceReport {
    pub effect_block_index: u8,
    pub custom_force_data_offset: u16,
    pub sample_count: u16,
}

impl HIDReport for SetCustomForceReport {
    const ID: ReportID = ReportID(ReportType::Output, 0x0E);
}

impl HIDReportOut for SetCustomForceReport {
    fn into_report(bytes: &[u8]) -> Option<Self> {
        Self::from_ram(&bytes[2..], *bytes.get(1)?)
    }
}

impl HIDReportRAM<4> for SetCustomForceReport {
    fn from_ram(ram: &[u8], effect_block_index: u8) -> Option<Self> {
        Some(Self {
            effect_block_index,
            custom_force_data_offset: u16::from_le_bytes([*ram.get(1)?, *ram.get(2)?]),
            sample_count: u16::from_le_bytes([*ram.get(3)?, *ram.get(4)?]),
        })
    }

    fn to_ram(&self) -> [u8; 4] {
        [
            self.custom_force_data_offset.to_le_bytes()[0],
            self.custom_force_data_offset.to_le_bytes()[1],
            self.sample_count.to_le_bytes()[0],
            self.sample_count.to_le_bytes()[1],
        ]
    }
}

// PID Pool Move Report
pub struct PIDPoolMoveReport {
    pub move_source: u16,
    pub move_destination: u16,
    pub move_length: u16,
}

impl HIDReport for PIDPoolMoveReport {
    const ID: ReportID = ReportID(ReportType::Output, 0x0F);
}

impl HIDReportOut for PIDPoolMoveReport {
    fn into_report(bytes: &[u8]) -> Option<Self> {
        Some(Self {
            move_source: u16::from_le_bytes([*bytes.get(1)?, *bytes.get(2)?]),
            move_destination: u16::from_le_bytes([*bytes.get(3)?, *bytes.get(4)?]),
            move_length: u16::from_le_bytes([*bytes.get(5)?, *bytes.get(6)?]),
        })
    }
}

// Create New Effect Report
#[derive(Clone, Copy)]
pub struct CreateNewEffectReport {
    pub effect_type: EffectType,
    pub byte_count: u16,
}

impl HIDReport for CreateNewEffectReport {
    const ID: ReportID = ReportID(ReportType::Feature, 0x01);
}

impl HIDReportOut for CreateNewEffectReport {
    fn into_report(bytes: &[u8]) -> Option<Self> {
        Some(Self {
            effect_type: EffectType::try_from(*bytes.get(1)?).ok()?,
            byte_count: u16::from_le_bytes([*bytes.get(2)?, *bytes.get(3)?]),
        })
    }
}

// PID Block Load Report
pub struct PIDBlockLoadReport {
    pub effect_block_index: u8,
    pub block_load_status: BlockLoadStatus,
    pub ram_pool_available: u16,
}

impl HIDReport for PIDBlockLoadReport {
    const ID: ReportID = ReportID(ReportType::Feature, 0x02);
}

impl HIDReportIn<5> for PIDBlockLoadReport {
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
    Full = 0x02,
    Error = 0x03,
}

// PID Pool Report
pub struct PIDPoolReport {
    pub ram_pool_size: u16,
    pub simultaneous_effects_max: u8,
    pub param_block_size_set_effect: u8,
    pub param_block_size_set_envelope: u8,
    pub param_block_size_set_condition: u8,
    pub param_block_size_set_periodic: u8,
    pub param_block_size_set_constant_force: u8,
    pub param_block_size_set_ramp_force: u8,
    pub param_block_size_set_custom_force: u8,
    pub device_managed_pool: bool,
    pub shared_parameter_blocks: bool,
    pub isochronous_enable: bool,
}

impl HIDReport for PIDPoolReport {
    const ID: ReportID = ReportID(ReportType::Feature, 0x03);
}

impl HIDReportIn<12> for PIDPoolReport {
    fn report_bytes(&self) -> [u8; 12] {
        [
            Self::ID.1,
            self.ram_pool_size.to_le_bytes()[0],
            self.ram_pool_size.to_le_bytes()[1],
            self.simultaneous_effects_max,
            self.param_block_size_set_effect,
            self.param_block_size_set_envelope,
            self.param_block_size_set_condition,
            self.param_block_size_set_periodic,
            self.param_block_size_set_constant_force,
            self.param_block_size_set_ramp_force,
            self.param_block_size_set_custom_force,
            bitflags(&[
                self.device_managed_pool,
                self.shared_parameter_blocks,
                self.isochronous_enable,
            ]),
        ]
    }
}
