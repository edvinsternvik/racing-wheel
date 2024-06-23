use crate::{
    descriptor::RACING_WHEEL_DESCRIPTOR,
    hid::{GetReportInWriter, ReportWriter},
    hid_device::{HIDDeviceType, HIDReport, HIDReportOut, HIDReportRAM, ReportID},
    reports::{
        BlockLoadStatus, CreateNewEffectReport, CustomForceDataReport, DeviceControl, DeviceGainReport, EffectOperation, EffectOperationReport, EffectType, JoystickReport, PIDBlockFreeReport, PIDBlockLoadReport, PIDDeviceControl, PIDPoolMoveReport, PIDPoolReport, PIDStateReport, SetConditionReport, SetConstantForceReport, SetCustomForceReport, SetEffectReport, SetEnvelopeReport, SetPeriodicReport, SetRampForceReport
    },
};
use usb_device::{bus::UsbBus, UsbError};

const MAX_EFFECT_REPORTS: usize = 16;

struct RAMPool<const N: usize> {
    buffer: [u8; N],
    allocated: usize,
    effects: [Option<EffectType>; MAX_EFFECT_REPORTS],
    next_effect_type: Option<EffectType>,
    device_managed: bool,
}

impl<const N: usize> RAMPool<N> {
    fn new() -> Self {
        Self {
            buffer: [0; N],
            allocated: MAX_EFFECT_REPORTS * SetEffectReport::RAM_SIZE,
            effects: [None; MAX_EFFECT_REPORTS],
            next_effect_type: None,
            device_managed: true,
        }
    }

    fn allocate(&mut self, length: usize) -> Result<usize, ()> {
        if self.allocated + length >= N {
            return Err(());
        }
        let address = self.allocated;
        self.allocated += length;
        Ok(address)
    }

    fn write_report<const M: usize>(
        &mut self,
        report: &impl HIDReportRAM<M>,
        address: usize,
    ) -> Result<(), ()> {
        if address + M > N {
            return Err(());
        }
        self.buffer[address..address + M].copy_from_slice(&report.to_ram());
        Ok(())
    }

    fn read_report<const M: usize, R: HIDReportRAM<M>>(
        &self,
        address: usize,
        effect_block_index: u8,
    ) -> Result<R, ()> {
        R::from_ram(&self.buffer[address..], effect_block_index).ok_or(())
    }
}

struct FixedSet<T, const N: usize> {
    array: [T; N],
    n: usize,
}

impl<T: Eq + Copy + Clone + Default, const N: usize> FixedSet<T, N> {
    fn new() -> Self {
        Self {
            array: [T::default(); N],
            n: 0,
        }
    }

    fn size(&self) -> usize {
        self.n
    }

    fn insert(&mut self, elem: T) -> bool {
        if self.n >= N || self.items().iter().any(|e| *e == elem) {
            return false;
        }
        self.array[self.n] = elem;
        self.n += 1;
        true
    }

    fn remove(&mut self, v: T) -> bool {
        for (i, item) in self.items().iter().enumerate() {
            if *item == v {
                self.array[i] = self.array[self.n - 1];
                self.n -= 1;
                return true;
            }
        }
        false
    }

    fn items(&self) -> &[T] {
        &self.array[0..self.n]
    }
}

const MAX_SIMULTANEOUS_EFFECTS: usize = 8;

pub struct RacingWheel {
    ram_pool: RAMPool<4096>,
    running_effects: FixedSet<u8, MAX_SIMULTANEOUS_EFFECTS>,
    actuators_enabled: bool,
    paused: bool,
    device_gain: u8,
}

impl RacingWheel {
    pub fn new() -> Self {
        RacingWheel {
            ram_pool: RAMPool::new(),
            running_effects: FixedSet::new(),
            actuators_enabled: false,
            paused: false,
            device_gain: 0,
        }
    }
}

impl RacingWheel {
    fn get_set_effect_report(&self, effect_block_index: u8) -> Result<SetEffectReport, UsbError> {
        // Check that the effect exists
        self.ram_pool
            .effects
            .get(effect_block_index as usize - 1)
            .ok_or(UsbError::Unsupported)?
            .ok_or(UsbError::Unsupported)?;

        // Read the Set Effect Report
        self.ram_pool
            .read_report(effect_address(effect_block_index), effect_block_index)
            .map_err(|_| UsbError::ParseError)
    }

    fn get_type_specific_block_offsets(
        &self,
        effect_block_index: u8,
    ) -> Result<[usize; 2], UsbError> {
        let set_effect_report = self.get_set_effect_report(effect_block_index)?;

        Ok([
            set_effect_report.type_specific_block_offset_instance_1 as usize,
            set_effect_report.type_specific_block_offset_instance_2 as usize,
        ])
    }
}

impl HIDDeviceType for RacingWheel {
    fn descriptor() -> &'static [u8] {
        RACING_WHEEL_DESCRIPTOR
    }

    fn get_report_request<B: UsbBus>(
        &mut self,
        report_id: ReportID,
        writer: GetReportInWriter<B>,
    ) -> Result<(), UsbError> {
        match report_id {
            PIDBlockLoadReport::ID => {
                let effects = self.ram_pool.effects.iter().enumerate();
                let index = effects.filter(|e| e.1.is_none()).next().map(|e| e.0);
                let effect_type = self.ram_pool.next_effect_type;
                let ram_pool_available =
                    (self.ram_pool.buffer.len() - self.ram_pool.allocated) as u16;

                self.ram_pool.next_effect_type = None;
                match (index, effect_type) {
                    (Some(index), Some(effect_type)) => {
                        self.ram_pool.effects[index] = Some(effect_type);

                        writer.accept(PIDBlockLoadReport {
                            effect_block_index: (index + 1) as u8,
                            block_load_status: BlockLoadStatus::Success,
                            ram_pool_available,
                        })
                    }
                    (None, _) => writer.accept(PIDBlockLoadReport {
                        effect_block_index: 0,
                        block_load_status: BlockLoadStatus::Full,
                        ram_pool_available,
                    }),
                    (_, None) => writer.accept(PIDBlockLoadReport {
                        effect_block_index: 0,
                        block_load_status: BlockLoadStatus::Error,
                        ram_pool_available,
                    }),
                }
            }
            PIDPoolReport::ID => writer.accept(PIDPoolReport {
                ram_pool_size: self.ram_pool.buffer.len() as u16,
                simultaneous_effects_max: MAX_SIMULTANEOUS_EFFECTS as u8,
                param_block_size_set_effect: SetEffectReport::RAM_SIZE as u8,
                param_block_size_set_envelope: SetEnvelopeReport::RAM_SIZE as u8,
                param_block_size_set_condition: SetConditionReport::RAM_SIZE as u8,
                param_block_size_set_periodic: SetPeriodicReport::RAM_SIZE as u8,
                param_block_size_set_constant_force: SetConstantForceReport::RAM_SIZE as u8,
                param_block_size_set_ramp_force: SetRampForceReport::RAM_SIZE as u8,
                param_block_size_set_custom_force: SetCustomForceReport::RAM_SIZE as u8,
                device_managed_pool: self.ram_pool.device_managed,
                shared_parameter_blocks: false,
            }),
            _ => Ok(()),
        }
    }

    fn report_request_out(
        &mut self,
        report_id: ReportID,
        data: &[u8],
    ) -> Result<Option<bool>, UsbError> {
        match report_id {
            SetEffectReport::ID => {
                let mut report = SetEffectReport::into_report(data).ok_or(UsbError::ParseError)?;
                let parameter_ram_sizes = get_parameter_ram_sizes(report.effect_type);
                let address = self
                    .ram_pool
                    .allocate(parameter_ram_sizes.iter().sum())
                    .map_err(|_| UsbError::BufferOverflow)?;

                report.type_specific_block_offset_instance_1 = address as u16;
                report.type_specific_block_offset_instance_2 =
                    (address + parameter_ram_sizes[0]) as u16;

                self.ram_pool
                    .write_report(&report, effect_address(report.effect_block_index))
                    .map_err(|_| UsbError::BufferOverflow)?;
                Ok(Some(true))
            }
            SetEnvelopeReport::ID => {
                let report = SetEnvelopeReport::into_report(data).ok_or(UsbError::ParseError)?;
                let address = self.get_type_specific_block_offsets(report.effect_block_index)?[1];

                self.ram_pool
                    .write_report(&report, address)
                    .map_err(|_| UsbError::BufferOverflow)?;
                Ok(Some(true))
            }
            SetConditionReport::ID => {
                let report = SetConditionReport::into_report(data).ok_or(UsbError::ParseError)?;
                let addresses = self.get_type_specific_block_offsets(report.effect_block_index)?;
                let address = *addresses
                    .get(report.parameter_block_offset as usize)
                    .ok_or(UsbError::ParseError)?;

                self.ram_pool
                    .write_report(&report, address)
                    .map_err(|_| UsbError::BufferOverflow)?;
                Ok(Some(true))
            }
            SetPeriodicReport::ID => {
                let report = SetPeriodicReport::into_report(data).ok_or(UsbError::ParseError)?;
                let address = self.get_type_specific_block_offsets(report.effect_block_index)?[0];

                self.ram_pool
                    .write_report(&report, address)
                    .map_err(|_| UsbError::BufferOverflow)?;
                Ok(Some(true))
            }
            SetConstantForceReport::ID => {
                let report =
                    SetConstantForceReport::into_report(data).ok_or(UsbError::ParseError)?;
                let address = self.get_type_specific_block_offsets(report.effect_block_index)?[0];

                self.ram_pool
                    .write_report(&report, address)
                    .map_err(|_| UsbError::BufferOverflow)?;
                Ok(Some(true))
            }
            SetRampForceReport::ID => {
                let report = SetRampForceReport::into_report(data).ok_or(UsbError::ParseError)?;
                let address = self.get_type_specific_block_offsets(report.effect_block_index)?[0];

                self.ram_pool
                    .write_report(&report, address)
                    .map_err(|_| UsbError::BufferOverflow)?;
                Ok(Some(true))
            }
            CustomForceDataReport::ID => {
                let report =
                    CustomForceDataReport::into_report(data).ok_or(UsbError::ParseError)?;
                let address = self.get_type_specific_block_offsets(report.effect_block_index)?[0];

                self.ram_pool
                    .write_report(&report, address)
                    .map_err(|_| UsbError::BufferOverflow)?;
                Ok(Some(true))
            }
            EffectOperationReport::ID => {
                let report =
                    EffectOperationReport::into_report(data).ok_or(UsbError::ParseError)?;
                match report.effect_operation {
                    EffectOperation::EffectStart => {
                        self.running_effects.insert(report.effect_block_index);
                    }
                    EffectOperation::EffectStartSolo => {
                        self.running_effects = FixedSet::new();
                        self.running_effects.insert(report.effect_block_index);
                    }
                    EffectOperation::EffectStop => {
                        self.running_effects.remove(report.effect_block_index);
                    }
                }

                Ok(Some(true))
            }
            PIDBlockFreeReport::ID => {
                let report = PIDBlockFreeReport::into_report(data).ok_or(UsbError::ParseError)?;
                let effect = self
                    .ram_pool
                    .effects
                    .get_mut((report.effect_block_index - 1) as usize)
                    .ok_or(UsbError::ParseError)?;
                *effect = None;
                Ok(Some(true))
            }
            PIDDeviceControl::ID => {
                let report = PIDDeviceControl::into_report(data).ok_or(UsbError::ParseError)?;
                match report.device_control {
                    DeviceControl::EnableActuators => self.actuators_enabled = true,
                    DeviceControl::DisableActuators => self.actuators_enabled = false,
                    DeviceControl::StopAllEffects => {}
                    DeviceControl::DeviceReset => {}
                    DeviceControl::DevicePause => self.paused = true,
                    DeviceControl::DeviceContinue => self.paused = false,
                }

                Ok(Some(true))
            }
            DeviceGainReport::ID => {
                let report = DeviceGainReport::into_report(data).ok_or(UsbError::ParseError)?;
                self.device_gain = report.device_gain;
                Ok(Some(true))
            }
            SetCustomForceReport::ID => {
                let _ = SetCustomForceReport::into_report(data).ok_or(UsbError::ParseError)?;
                Ok(Some(true))
            }
            PIDPoolMoveReport::ID => {
                let _ = PIDPoolMoveReport::into_report(data).ok_or(UsbError::ParseError)?;
                Ok(Some(true))
            }
            CreateNewEffectReport::ID => {
                let report =
                    CreateNewEffectReport::into_report(data).ok_or(UsbError::ParseError)?;
                self.ram_pool.next_effect_type = Some(report.effect_type);
                Ok(Some(true))
            }
            _ => Ok(None),
        }
    }

    fn send_input_reports<B: UsbBus>(&mut self, writer: ReportWriter<B>) -> Result<(), UsbError> {
        writer.write_report(JoystickReport {
            buttons: [false; 8],
            joystick_x: 0,
            joystick_y: 0,
        })?;

        writer.write_report(PIDStateReport {
            device_paused: self.paused,
            actuators_enabled: self.actuators_enabled,
            safety_switch: false,
            actuators_override_switch: false,
            actuator_power: false,
            effect_playing: self.running_effects.size() > 0,
            effect_block_index: 0,
        })?;

        Ok(())
    }
}

fn effect_address(effect_block_index: u8) -> usize {
    (effect_block_index - 1) as usize * SetEffectReport::RAM_SIZE
}

fn get_parameter_ram_sizes(effect_type: EffectType) -> [usize; 2] {
    match effect_type {
        EffectType::ConstantForce => [
            SetConstantForceReport::RAM_SIZE,
            SetEnvelopeReport::RAM_SIZE,
        ],
        EffectType::Ramp => [SetRampForceReport::RAM_SIZE, SetEnvelopeReport::RAM_SIZE],
        EffectType::Square => [SetPeriodicReport::RAM_SIZE, SetEnvelopeReport::RAM_SIZE],
        EffectType::Sine => [SetPeriodicReport::RAM_SIZE, SetEnvelopeReport::RAM_SIZE],
        EffectType::Triangle => [SetPeriodicReport::RAM_SIZE, SetEnvelopeReport::RAM_SIZE],
        EffectType::SawtoothUp => [SetPeriodicReport::RAM_SIZE, SetEnvelopeReport::RAM_SIZE],
        EffectType::SawtoothDown => [SetPeriodicReport::RAM_SIZE, SetEnvelopeReport::RAM_SIZE],
        EffectType::Spring => [SetConditionReport::RAM_SIZE, SetConditionReport::RAM_SIZE],
        EffectType::Damper => [SetConditionReport::RAM_SIZE, SetConditionReport::RAM_SIZE],
        EffectType::Inertia => [SetConditionReport::RAM_SIZE, SetConditionReport::RAM_SIZE],
        EffectType::Friction => [SetConditionReport::RAM_SIZE, SetConditionReport::RAM_SIZE],
        EffectType::CustomForceData => [SetCustomForceReport::RAM_SIZE, 0],
    }
}
