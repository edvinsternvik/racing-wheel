use super::descriptor::LOGICAL_MAXIMUM;
use crate::{
    config::Config,
    misc::{bitflag, bitflags, bits},
};
use core::{
    convert::{TryFrom, TryInto},
    ops::{Deref, DerefMut},
};
use force_feedback::reports::*;
use usb_hid_device::hid_device::{
    HIDReport, HIDReportIn, HIDReportOut, HIDReportRAM, ReportID, ReportType,
};

pub struct Report<T>(pub T);

impl<T> Deref for Report<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Report<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl HIDReport for Report<RacingWheelState> {
    const ID: ReportID = ReportID(ReportType::Input, 0x01);
}

impl HIDReportIn<6> for Report<RacingWheelState> {
    fn report_bytes(&self) -> [u8; 6] {
        [
            Self::ID.1,
            bitflags(&self.buttons),
            f32_to_2_bytes(self.steering)[0],
            f32_to_2_bytes(self.steering)[1],
            f32_to_2_bytes(self.throttle)[0],
            f32_to_2_bytes(self.throttle)[1],
        ]
    }
}

impl HIDReport for Report<PIDState> {
    const ID: ReportID = ReportID(ReportType::Input, 0x02);
}

impl HIDReportIn<3> for Report<PIDState> {
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

impl HIDReport for Report<SetEffect> {
    const ID: ReportID = ReportID(ReportType::Output, 0x01);
}

impl HIDReportOut for Report<SetEffect> {
    fn into_report(bytes: &[u8]) -> Option<Self> {
        Self::from_ram(&bytes[2..], *bytes.get(1)?)
    }
}

impl HIDReportRAM<19> for Report<SetEffect> {
    fn from_ram(ram: &[u8], effect_block_index: u8) -> Option<Self> {
        let duration = u16::from_le_bytes([*ram.get(1)?, *ram.get(2)?]);
        let sample_period = u16::from_le_bytes([*ram.get(5)?, *ram.get(6)?]);
        Some(Report(SetEffect {
            effect_block_index,
            effect_type: EffectType::try_from(*ram.get(0)?).ok()?,
            duration: if duration == 0 || duration == u16::MAX {
                None
            } else {
                Some(duration)
            },
            trigger_repeat_interval: u16::from_le_bytes([*ram.get(3)?, *ram.get(4)?]),
            sample_period: if sample_period == 0 {
                None
            } else {
                Some(sample_period)
            },
            gain: f32_from_2_bytes(&ram[7..])?,
            trigger_button: *ram.get(9)?,
            axis_x_enable: bitflag(*ram.get(10)?, 0),
            axis_y_enable: bitflag(*ram.get(10)?, 1),
            direction_enable: bitflag(*ram.get(10)?, 2),
            direction_instance_1: *ram.get(11)?,
            direction_instance_2: *ram.get(12)?,
            start_delay: u16::from_le_bytes([*ram.get(13)?, *ram.get(14)?]),
            type_specific_block_offset_instance_1: u16::from_le_bytes([
                *ram.get(15)?,
                *ram.get(16)?,
            ]),
            type_specific_block_offset_instance_2: u16::from_le_bytes([
                *ram.get(17)?,
                *ram.get(18)?,
            ]),
        }))
    }

    fn to_ram(&self) -> [u8; 19] {
        [
            self.effect_type as u8,
            self.duration.unwrap_or_default().to_le_bytes()[0],
            self.duration.unwrap_or_default().to_le_bytes()[1],
            self.trigger_repeat_interval.to_le_bytes()[0],
            self.trigger_repeat_interval.to_le_bytes()[1],
            self.sample_period.unwrap_or_default().to_le_bytes()[0],
            self.sample_period.unwrap_or_default().to_le_bytes()[1],
            ((self.gain * 10_000.0) as i16).to_le_bytes()[0],
            ((self.gain * 10_000.0) as i16).to_le_bytes()[1],
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

impl HIDReport for Report<SetEnvelope> {
    const ID: ReportID = ReportID(ReportType::Output, 0x02);
}

impl HIDReportOut for Report<SetEnvelope> {
    fn into_report(bytes: &[u8]) -> Option<Self> {
        Self::from_ram(&bytes[2..], *bytes.get(1)?)
    }
}

impl HIDReportRAM<12> for Report<SetEnvelope> {
    fn from_ram(ram: &[u8], effect_block_index: u8) -> Option<Self> {
        Some(Report(SetEnvelope {
            effect_block_index,
            attack_level: f32_from_2_bytes(&ram[0..])?,
            fade_level: f32_from_2_bytes(&ram[2..])?,
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
        }))
    }

    fn to_ram(&self) -> [u8; 12] {
        [
            ((self.attack_level * 10_000.0) as i16).to_le_bytes()[0],
            ((self.attack_level * 10_000.0) as i16).to_le_bytes()[1],
            ((self.fade_level * 10_000.0) as i16).to_le_bytes()[0],
            ((self.fade_level * 10_000.0) as i16).to_le_bytes()[1],
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

impl HIDReport for Report<SetCondition> {
    const ID: ReportID = ReportID(ReportType::Output, 0x03);
}

impl HIDReportOut for Report<SetCondition> {
    fn into_report(bytes: &[u8]) -> Option<Self> {
        Self::from_ram(&bytes[2..], *bytes.get(1)?)
    }
}

impl HIDReportRAM<13> for Report<SetCondition> {
    fn from_ram(ram: &[u8], effect_block_index: u8) -> Option<Self> {
        Some(Report(SetCondition {
            effect_block_index,
            parameter_block_offset: bits(*ram.get(0)?, 0, 4),
            type_specific_block_offset_instance_1: bits(*ram.get(0)?, 4, 2),
            type_specific_block_offset_instance_2: bits(*ram.get(0)?, 6, 2),
            cp_offset: f32_from_2_bytes(&ram[1..])?,
            positive_coefficient: f32_from_2_bytes(&ram[3..])?,
            negative_coefficient: f32_from_2_bytes(&ram[5..])?,
            positive_saturation: f32_from_2_bytes(&ram[7..])?,
            negative_saturation: f32_from_2_bytes(&ram[9..])?,
            dead_band: f32_from_2_bytes(&ram[11..])?,
        }))
    }

    fn to_ram(&self) -> [u8; 13] {
        [
            (self.parameter_block_offset & 0b1111) << 0
                | (self.type_specific_block_offset_instance_1 & 0b11) << 4
                | (self.type_specific_block_offset_instance_2 & 0b11) << 6,
            ((self.cp_offset * 10_000.0) as i16).to_le_bytes()[0],
            ((self.cp_offset * 10_000.0) as i16).to_le_bytes()[1],
            ((self.positive_coefficient * 10_000.0) as i16).to_le_bytes()[0],
            ((self.positive_coefficient * 10_000.0) as i16).to_le_bytes()[1],
            ((self.negative_coefficient * 10_000.0) as i16).to_le_bytes()[0],
            ((self.negative_coefficient * 10_000.0) as i16).to_le_bytes()[1],
            ((self.positive_saturation * 10_000.0) as i16).to_le_bytes()[0],
            ((self.positive_saturation * 10_000.0) as i16).to_le_bytes()[1],
            ((self.negative_saturation * 10_000.0) as i16).to_le_bytes()[0],
            ((self.negative_saturation * 10_000.0) as i16).to_le_bytes()[1],
            ((self.dead_band * 10_000.0) as i16).to_le_bytes()[0],
            ((self.dead_band * 10_000.0) as i16).to_le_bytes()[1],
        ]
    }
}

impl HIDReport for Report<SetPeriodic> {
    const ID: ReportID = ReportID(ReportType::Output, 0x04);
}

impl HIDReportOut for Report<SetPeriodic> {
    fn into_report(bytes: &[u8]) -> Option<Self> {
        Self::from_ram(&bytes[2..], *bytes.get(1)?)
    }
}

impl HIDReportRAM<10> for Report<SetPeriodic> {
    fn from_ram(ram: &[u8], effect_block_index: u8) -> Option<Self> {
        Some(Report(SetPeriodic {
            effect_block_index,
            magnitude: f32_from_2_bytes(&ram[0..])?,
            offset: f32_from_2_bytes(&ram[2..])?,
            phase: u16::from_le_bytes([*ram.get(4)?, *ram.get(5)?]),
            period: u32::from_le_bytes([*ram.get(6)?, *ram.get(7)?, *ram.get(8)?, *ram.get(9)?]),
        }))
    }

    fn to_ram(&self) -> [u8; 10] {
        [
            ((self.magnitude * 10_000.0) as i16).to_le_bytes()[0],
            ((self.magnitude * 10_000.0) as i16).to_le_bytes()[1],
            ((self.offset * 10_000.0) as i16).to_le_bytes()[0],
            ((self.offset * 10_000.0) as i16).to_le_bytes()[1],
            self.phase.to_le_bytes()[0],
            self.phase.to_le_bytes()[1],
            self.period.to_le_bytes()[0],
            self.period.to_le_bytes()[1],
            self.period.to_le_bytes()[2],
            self.period.to_le_bytes()[3],
        ]
    }
}

impl HIDReport for Report<SetConstantForce> {
    const ID: ReportID = ReportID(ReportType::Output, 0x05);
}

impl HIDReportOut for Report<SetConstantForce> {
    fn into_report(bytes: &[u8]) -> Option<Self> {
        Self::from_ram(&bytes[2..], *bytes.get(1)?)
    }
}

impl HIDReportRAM<2> for Report<SetConstantForce> {
    fn from_ram(ram: &[u8], effect_block_index: u8) -> Option<Self> {
        Some(Report(SetConstantForce {
            effect_block_index,
            magnitude: f32_from_2_bytes(&ram[0..])?,
        }))
    }

    fn to_ram(&self) -> [u8; 2] {
        [
            ((self.magnitude * 10_000.0) as i16).to_le_bytes()[0],
            ((self.magnitude * 10_000.0) as i16).to_le_bytes()[1],
        ]
    }
}

impl HIDReport for Report<SetRampForce> {
    const ID: ReportID = ReportID(ReportType::Output, 0x06);
}

impl HIDReportOut for Report<SetRampForce> {
    fn into_report(bytes: &[u8]) -> Option<Self> {
        Self::from_ram(&bytes[2..], *bytes.get(1)?)
    }
}

impl HIDReportRAM<4> for Report<SetRampForce> {
    fn from_ram(ram: &[u8], effect_block_index: u8) -> Option<Self> {
        Some(Report(SetRampForce {
            effect_block_index,
            ramp_start: f32_from_2_bytes(&ram[0..])?,
            ramp_end: f32_from_2_bytes(&ram[2..])?,
        }))
    }

    fn to_ram(&self) -> [u8; 4] {
        [
            ((self.ramp_start * 10_000.0) as i16).to_le_bytes()[0],
            ((self.ramp_start * 10_000.0) as i16).to_le_bytes()[1],
            ((self.ramp_end * 10_000.0) as i16).to_le_bytes()[0],
            ((self.ramp_end * 10_000.0) as i16).to_le_bytes()[1],
        ]
    }
}

impl HIDReport for Report<CustomForceData> {
    const ID: ReportID = ReportID(ReportType::Output, 0x07);
}

impl HIDReportOut for Report<CustomForceData> {
    fn into_report(bytes: &[u8]) -> Option<Self> {
        Self::from_ram(&bytes[2..], *bytes.get(1)?)
    }
}

impl HIDReportRAM<15> for Report<CustomForceData> {
    fn from_ram(ram: &[u8], effect_block_index: u8) -> Option<Self> {
        Some(Report(CustomForceData {
            effect_block_index,
            custom_force_data_offset: u16::from_le_bytes([*ram.get(0)?, *ram.get(1)?]),
            byte_count: *ram.get(2)?,
            custom_force_data: ram.get(3..(3 + 12))?.try_into().unwrap_or_default(),
        }))
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

impl HIDReport for Report<DownloadForceSample> {
    const ID: ReportID = ReportID(ReportType::Output, 0x08);
}

impl HIDReportOut for Report<DownloadForceSample> {
    fn into_report(bytes: &[u8]) -> Option<Self> {
        Some(Report(DownloadForceSample {
            steering: *bytes.get(1)? as i8,
            throttle: *bytes.get(2)?,
        }))
    }
}

impl HIDReport for Report<SetEffectOperation> {
    const ID: ReportID = ReportID(ReportType::Output, 0x0A);
}

impl HIDReportOut for Report<SetEffectOperation> {
    fn into_report(bytes: &[u8]) -> Option<Self> {
        Some(Report(SetEffectOperation {
            effect_block_index: *bytes.get(1)?,
            effect_operation: EffectOperation::try_from(*bytes.get(2)?).ok()?,
            loop_count: *bytes.get(3)?,
        }))
    }
}

impl HIDReport for Report<PIDBlockFree> {
    const ID: ReportID = ReportID(ReportType::Output, 0x0B);
}

impl HIDReportOut for Report<PIDBlockFree> {
    fn into_report(bytes: &[u8]) -> Option<Self> {
        Some(Report(PIDBlockFree {
            effect_block_index: *bytes.get(1)?,
        }))
    }
}

impl HIDReport for Report<PIDDeviceControl> {
    const ID: ReportID = ReportID(ReportType::Output, 0x0C);
}

impl HIDReportOut for Report<PIDDeviceControl> {
    fn into_report(bytes: &[u8]) -> Option<Self> {
        Some(Report(PIDDeviceControl {
            device_control: DeviceControl::try_from(*bytes.get(1)?).ok()?,
        }))
    }
}
impl HIDReport for Report<DeviceGain> {
    const ID: ReportID = ReportID(ReportType::Output, 0x0D);
}

impl HIDReportOut for Report<DeviceGain> {
    fn into_report(bytes: &[u8]) -> Option<Self> {
        Some(Report(DeviceGain {
            device_gain: f32_from_2_bytes(&bytes[1..])?,
        }))
    }
}

impl HIDReport for Report<SetCustomForce> {
    const ID: ReportID = ReportID(ReportType::Output, 0x0E);
}

impl HIDReportOut for Report<SetCustomForce> {
    fn into_report(bytes: &[u8]) -> Option<Self> {
        Self::from_ram(&bytes[2..], *bytes.get(1)?)
    }
}

impl HIDReportRAM<4> for Report<SetCustomForce> {
    fn from_ram(ram: &[u8], effect_block_index: u8) -> Option<Self> {
        Some(Report(SetCustomForce {
            effect_block_index,
            custom_force_data_offset: u16::from_le_bytes([*ram.get(1)?, *ram.get(2)?]),
            sample_count: u16::from_le_bytes([*ram.get(3)?, *ram.get(4)?]),
        }))
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

impl HIDReport for Report<PIDPoolMove> {
    const ID: ReportID = ReportID(ReportType::Output, 0x0F);
}

impl HIDReportOut for Report<PIDPoolMove> {
    fn into_report(bytes: &[u8]) -> Option<Self> {
        Some(Report(PIDPoolMove {
            move_source: u16::from_le_bytes([*bytes.get(1)?, *bytes.get(2)?]),
            move_destination: u16::from_le_bytes([*bytes.get(3)?, *bytes.get(4)?]),
            move_length: u16::from_le_bytes([*bytes.get(5)?, *bytes.get(6)?]),
        }))
    }
}

impl HIDReport for Report<CreateNewEffect> {
    const ID: ReportID = ReportID(ReportType::Feature, 0x01);
}

impl HIDReportOut for Report<CreateNewEffect> {
    fn into_report(bytes: &[u8]) -> Option<Self> {
        Some(Report(CreateNewEffect {
            effect_type: EffectType::try_from(*bytes.get(1)?).ok()?,
            byte_count: u16::from_le_bytes([*bytes.get(2)?, *bytes.get(3)?]),
        }))
    }
}

impl HIDReport for Report<PIDBlockLoad> {
    const ID: ReportID = ReportID(ReportType::Feature, 0x02);
}

impl HIDReportIn<5> for Report<PIDBlockLoad> {
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

impl HIDReport for Report<PIDPool> {
    const ID: ReportID = ReportID(ReportType::Feature, 0x03);
}

impl HIDReportIn<12> for Report<PIDPool> {
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

impl HIDReport for Report<Config> {
    const ID: ReportID = ReportID(ReportType::Feature, 0x04);
}

impl HIDReportOut for Report<Config> {
    fn into_report(bytes: &[u8]) -> Option<Self> {
        Some(Report(Config {
            gain: f32::from_le_bytes([
                *bytes.get(1)?,
                *bytes.get(2)?,
                *bytes.get(3)?,
                *bytes.get(4)?,
            ]),
            max_rotation: u16::from_le_bytes([*bytes.get(5)?, *bytes.get(6)?]),
            motor_max: f32::from_le_bytes([
                *bytes.get(7)?,
                *bytes.get(8)?,
                *bytes.get(9)?,
                *bytes.get(10)?,
            ]),
            motor_deadband: f32::from_le_bytes([
                *bytes.get(11)?,
                *bytes.get(12)?,
                *bytes.get(13)?,
                *bytes.get(14)?,
            ]),
        }))
    }
}

impl HIDReportIn<15> for Report<Config> {
    fn report_bytes(&self) -> [u8; 15] {
        [
            Self::ID.1,
            f32::to_le_bytes(self.gain)[0],
            f32::to_le_bytes(self.gain)[1],
            f32::to_le_bytes(self.gain)[2],
            f32::to_le_bytes(self.gain)[3],
            u16::to_le_bytes(self.max_rotation)[0],
            u16::to_le_bytes(self.max_rotation)[1],
            f32::to_le_bytes(self.motor_max)[0],
            f32::to_le_bytes(self.motor_max)[1],
            f32::to_le_bytes(self.motor_max)[2],
            f32::to_le_bytes(self.motor_max)[3],
            f32::to_le_bytes(self.motor_deadband)[0],
            f32::to_le_bytes(self.motor_deadband)[1],
            f32::to_le_bytes(self.motor_deadband)[2],
            f32::to_le_bytes(self.motor_deadband)[3],
        ]
    }
}

fn f32_from_2_bytes(bytes: &[u8]) -> Option<f32> {
    Some(i16::from_le_bytes([*bytes.get(0)?, *bytes.get(1)?]) as f32 / LOGICAL_MAXIMUM as f32)
}

fn f32_to_2_bytes(value: f32) -> [u8; 2] {
    ((value * LOGICAL_MAXIMUM as f32) as i16).to_le_bytes()
}
