use fixed_num::Fixed16;

pub const FORCE_LOGICAL_MAX: i32 = 10_000;
pub const STEERING_LOGICAL_MAX: i16 = 10_000;
pub type FixedSteering = Fixed16<{ STEERING_LOGICAL_MAX as u64 }>;
pub type FixedFFB = Fixed16<{ FORCE_LOGICAL_MAX as u64 }>;

// Racing wheel report
#[derive(Default, Clone)]
pub struct RacingWheelState {
    pub buttons: [bool; 8],
    pub steering: FixedSteering,
    pub throttle: FixedFFB,
}

// PID State Report
#[derive(Default, Clone)]
pub struct PIDState {
    pub device_paused: bool,
    pub actuators_enabled: bool,
    pub safety_switch: bool,
    pub actuators_override_switch: bool,
    pub actuator_power: bool,
    pub effect_playing: bool,
    pub effect_block_index: u8,
}

// Set Effect Report
#[derive(Clone, Copy)]
pub struct SetEffect {
    pub effect_block_index: u8,
    pub effect_type: EffectType,
    pub duration: Option<u16>,
    pub trigger_repeat_interval: u16,
    pub sample_period: Option<u16>,
    pub gain: FixedFFB,
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
#[derive(Clone, Copy)]
pub struct SetEnvelope {
    pub effect_block_index: u8,
    pub attack_level: FixedFFB,
    pub fade_level: FixedFFB,
    pub attack_time: u32,
    pub fade_time: u32,
}

// Set Condition Report
#[derive(Clone, Copy)]
pub struct SetCondition {
    pub effect_block_index: u8,
    pub parameter_block_offset: u8,
    pub type_specific_block_offset_instance_1: u8,
    pub type_specific_block_offset_instance_2: u8,
    pub cp_offset: FixedFFB,
    pub positive_coefficient: FixedFFB,
    pub negative_coefficient: FixedFFB,
    pub positive_saturation: FixedFFB,
    pub negative_saturation: FixedFFB,
    pub dead_band: FixedFFB,
}

// Set Periodic Report
#[derive(Clone, Copy)]
pub struct SetPeriodic {
    pub effect_block_index: u8,
    pub magnitude: FixedFFB,
    pub offset: FixedFFB,
    pub phase: u16,
    pub period: u32,
}

// Set Constant Force Report
#[derive(Clone, Copy)]
pub struct SetConstantForce {
    pub effect_block_index: u8,
    pub magnitude: FixedFFB,
}

// Set Ramp Force Report
#[derive(Clone, Copy)]
pub struct SetRampForce {
    pub effect_block_index: u8,
    pub ramp_start: FixedFFB,
    pub ramp_end: FixedFFB,
}

// Custom Force Data Report
#[derive(Clone, Copy)]
pub struct CustomForceData {
    pub effect_block_index: u8,
    pub custom_force_data_offset: u16,
    pub byte_count: u8,
    pub custom_force_data: [u8; 12],
}

// Download Force Sample
#[derive(Clone, Copy)]
pub struct DownloadForceSample {
    pub steering: i8,
    pub throttle: u8,
}

// Effect Operation Report
#[derive(Clone, Copy)]
pub struct SetEffectOperation {
    pub effect_block_index: u8,
    pub effect_operation: EffectOperation,
    pub loop_count: u8,
}

#[derive(Clone, Copy)]
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
#[derive(Clone, Copy)]
pub struct PIDBlockFree {
    pub effect_block_index: u8,
}

// PID Device Control
#[derive(Clone, Copy)]
pub struct PIDDeviceControl {
    pub device_control: DeviceControl,
}

#[derive(Clone, Copy)]
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
#[derive(Clone, Copy)]
pub struct DeviceGain {
    pub device_gain: FixedFFB,
}

// Set Custom Force Report
#[derive(Clone, Copy)]
pub struct SetCustomForce {
    pub effect_block_index: u8,
    pub custom_force_data_offset: u16,
    pub sample_count: u16,
}

// PID Pool Move Report
#[derive(Clone, Copy)]
pub struct PIDPoolMove {
    pub move_source: u16,
    pub move_destination: u16,
    pub move_length: u16,
}

// Create New Effect Report
#[derive(Clone, Copy)]
pub struct CreateNewEffect {
    pub effect_type: EffectType,
    pub byte_count: u16,
}

// PID Block Load Report
#[derive(Clone, Copy)]
pub struct PIDBlockLoad {
    pub effect_block_index: u8,
    pub block_load_status: BlockLoadStatus,
    pub ram_pool_available: u16,
}

#[derive(Clone, Copy)]
pub enum BlockLoadStatus {
    Success = 0x01,
    Full = 0x02,
    Error = 0x03,
}

// PID Pool Report
#[derive(Clone, Copy)]
pub struct PIDPool {
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

// Set Configuration Report
#[derive(Clone, Copy, Default)]
pub struct SetConfig {
    pub gain: FixedFFB,
}
