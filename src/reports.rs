use crate::hid::{HIDReport, ReportID, ReportType};

// Create New Effect Report
pub struct CreateNewEffectReport {
    pub effect_type: u8,
    pub byte_count: u8,
}

impl HIDReport<3> for CreateNewEffectReport {
    const ID: ReportID = ReportID(ReportType::Feature, 0x01);
    fn report_bytes(&self) -> [u8; 3] {
        return [
            Self::ID.1,
            self.effect_type,
            self.byte_count,
        ];
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
}

impl HIDReport<3> for PIDBlockLoadReport {
    const ID: ReportID = ReportID(ReportType::Feature, 0x02);
    fn report_bytes(&self) -> [u8; 3] {
        return [
            Self::ID.1,
            self.effect_block_index,
            self.block_load_status as u8,
        ];
    }
}

// PID Pool Report
pub struct PIDPoolReport {
    pub ram_pool_size: u16,
    pub simultaneous_effects_max: u8,
    pub device_managed_pool: bool,
    pub shared_parameter_blocks: bool,
}

impl HIDReport<5> for PIDPoolReport {
    const ID: ReportID = ReportID(ReportType::Feature, 0x03);
    fn report_bytes(&self) -> [u8; 5] {
        return [
            Self::ID.1,
            self.ram_pool_size.to_le_bytes()[0],
            self.ram_pool_size.to_le_bytes()[1],
            self.simultaneous_effects_max,
            self.device_managed_pool as u8 | (self.shared_parameter_blocks as u8) << 1,
        ];
    }
}
