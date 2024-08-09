use crate::{
    misc::{bitflag, bitflags, bits},
    usb::hid_device::{HIDReport, HIDReportIn, HIDReportOut, HIDReportRAM, ReportID, ReportType},
};
use core::convert::{TryFrom, TryInto};
use force_feedback::reports::*;

impl HIDReport for RacingWheelReport {
    const ID: ReportID = ReportID(ReportType::Input, 0x01);
}

impl HIDReportIn<6> for RacingWheelReport {
    fn report_bytes(&self) -> [u8; 6] {
        [
            Self::ID.1,
            bitflags(&self.buttons),
            self.steering.value().to_le_bytes()[0],
            self.steering.value().to_le_bytes()[1],
            self.throttle.value().to_le_bytes()[0],
            self.throttle.value().to_le_bytes()[1],
        ]
    }
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

impl HIDReport for SetEffectReport {
    const ID: ReportID = ReportID(ReportType::Output, 0x01);
}

impl HIDReportOut for SetEffectReport {
    fn into_report(bytes: &[u8]) -> Option<Self> {
        Self::from_ram(&bytes[2..], *bytes.get(1)?)
    }
}

impl HIDReportRAM<19> for SetEffectReport {
    fn from_ram(ram: &[u8], effect_block_index: u8) -> Option<Self> {
        let duration = u16::from_le_bytes([*ram.get(1)?, *ram.get(2)?]);
        let sample_period = u16::from_le_bytes([*ram.get(5)?, *ram.get(6)?]);
        Some(Self {
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
            gain: i16::from_le_bytes([*ram.get(7)?, *ram.get(8)?]).into(),
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
        })
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
            self.gain.value().to_le_bytes()[0],
            self.gain.value().to_le_bytes()[1],
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
            attack_level: i16::from_le_bytes([*ram.get(0)?, *ram.get(1)?]).into(),
            fade_level: i16::from_le_bytes([*ram.get(2)?, *ram.get(3)?]).into(),
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
            self.attack_level.value().to_le_bytes()[0],
            self.attack_level.value().to_le_bytes()[1],
            self.fade_level.value().to_le_bytes()[0],
            self.fade_level.value().to_le_bytes()[1],
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
            cp_offset: i16::from_le_bytes([*ram.get(1)?, *ram.get(2)?]).into(),
            positive_coefficient: i16::from_le_bytes([*ram.get(3)?, *ram.get(4)?]).into(),
            negative_coefficient: i16::from_le_bytes([*ram.get(5)?, *ram.get(6)?]).into(),
            positive_saturation: i16::from_le_bytes([*ram.get(7)?, *ram.get(8)?]).into(),
            negative_saturation: i16::from_le_bytes([*ram.get(9)?, *ram.get(10)?]).into(),
            dead_band: i16::from_le_bytes([*ram.get(11)?, *ram.get(12)?]).into(),
        })
    }

    fn to_ram(&self) -> [u8; 13] {
        [
            (self.parameter_block_offset & 0b1111) << 0
                | (self.type_specific_block_offset_instance_1 & 0b11) << 4
                | (self.type_specific_block_offset_instance_2 & 0b11) << 6,
            self.cp_offset.value().to_le_bytes()[0],
            self.cp_offset.value().to_le_bytes()[1],
            self.positive_coefficient.value().to_le_bytes()[0],
            self.positive_coefficient.value().to_le_bytes()[1],
            self.negative_coefficient.value().to_le_bytes()[0],
            self.negative_coefficient.value().to_le_bytes()[1],
            self.positive_saturation.value().to_le_bytes()[0],
            self.positive_saturation.value().to_le_bytes()[1],
            self.negative_saturation.value().to_le_bytes()[0],
            self.negative_saturation.value().to_le_bytes()[1],
            self.dead_band.value().to_le_bytes()[0],
            self.dead_band.value().to_le_bytes()[1],
        ]
    }
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
            magnitude: i16::from_le_bytes([*ram.get(0)?, *ram.get(1)?]).into(),
            offset: i16::from_le_bytes([*ram.get(2)?, *ram.get(3)?]).into(),
            phase: u16::from_le_bytes([*ram.get(4)?, *ram.get(5)?]),
            period: u32::from_le_bytes([*ram.get(6)?, *ram.get(7)?, *ram.get(8)?, *ram.get(9)?]),
        })
    }

    fn to_ram(&self) -> [u8; 10] {
        [
            self.magnitude.value().to_le_bytes()[0],
            self.magnitude.value().to_le_bytes()[1],
            self.offset.value().to_le_bytes()[0],
            self.offset.value().to_le_bytes()[1],
            self.phase.to_le_bytes()[0],
            self.phase.to_le_bytes()[1],
            self.period.to_le_bytes()[0],
            self.period.to_le_bytes()[1],
            self.period.to_le_bytes()[2],
            self.period.to_le_bytes()[3],
        ]
    }
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
            magnitude: i16::from_le_bytes([*ram.get(0)?, *ram.get(1)?]).into(),
        })
    }

    fn to_ram(&self) -> [u8; 2] {
        [
            self.magnitude.value().to_le_bytes()[0],
            self.magnitude.value().to_le_bytes()[1],
        ]
    }
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
            ramp_start: i16::from_le_bytes([*ram.get(0)?, *ram.get(1)?]).into(),
            ramp_end: i16::from_le_bytes([*ram.get(2)?, *ram.get(3)?]).into(),
        })
    }

    fn to_ram(&self) -> [u8; 4] {
        [
            self.ramp_start.value().to_le_bytes()[0],
            self.ramp_start.value().to_le_bytes()[1],
            self.ramp_end.value().to_le_bytes()[0],
            self.ramp_end.value().to_le_bytes()[1],
        ]
    }
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

impl HIDReport for DownloadForceSample {
    const ID: ReportID = ReportID(ReportType::Output, 0x08);
}

impl HIDReportOut for DownloadForceSample {
    fn into_report(bytes: &[u8]) -> Option<Self> {
        Some(Self {
            steering: *bytes.get(1)? as i8,
            throttle: *bytes.get(2)?,
        })
    }
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
impl HIDReport for DeviceGainReport {
    const ID: ReportID = ReportID(ReportType::Output, 0x0D);
}

impl HIDReportOut for DeviceGainReport {
    fn into_report(bytes: &[u8]) -> Option<Self> {
        Some(Self {
            device_gain: i16::from_le_bytes([*bytes.get(1)?, *bytes.get(2)?]).into(),
        })
    }
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

impl HIDReport for SetConfigReport {
    const ID: ReportID = ReportID(ReportType::Feature, 0x04);
}

impl HIDReportOut for SetConfigReport {
    fn into_report(bytes: &[u8]) -> Option<Self> {
        Some(Self {
            gain: i16::from_le_bytes([*bytes.get(1)?, *bytes.get(2)?]).into(),
        })
    }
}
