use crate::{
    hid_device::HIDReportRAM,
    reports::{EffectType, SetEffectReport},
};

pub struct RAMPool<const N: usize, const MAX_EFFECTS: usize> {
    buffer: [u8; N],
    allocated: usize,
    effects: [Option<EffectType>; MAX_EFFECTS],
}

impl<const N: usize, const MAX_EFFECTS: usize> RAMPool<N, MAX_EFFECTS> {
    pub fn new() -> Self {
        Self {
            buffer: [0; N],
            allocated: MAX_EFFECTS * SetEffectReport::RAM_SIZE,
            effects: [None; MAX_EFFECTS],
        }
    }

    pub fn allocate(&mut self, length: usize) -> Result<usize, ()> {
        if self.allocated + length >= N {
            return Err(());
        }
        let address = self.allocated;
        self.allocated += length;
        Ok(address)
    }

    pub fn write_report<const M: usize>(
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

    pub fn read_report<const M: usize, R: HIDReportRAM<M>>(
        &self,
        address: usize,
        effect_block_index: u8,
    ) -> Result<R, ()> {
        R::from_ram(&self.buffer[address..], effect_block_index).ok_or(())
    }

    pub fn new_effect(&mut self, effect_type: EffectType) -> Option<u8> {
        let effects = self.effects.iter().enumerate();
        let index = effects.filter(|e| e.1.is_none()).next().map(|e| e.0)?;
        *self.effects.get_mut(index)? = Some(effect_type);
        Some(index as u8 + 1)
    }

    pub fn free_effect(&mut self, effect_block_index: u8) -> Result<(), ()> {
        let effect = self.effects.get_mut((effect_block_index - 1) as usize).ok_or(())?;
        *effect = None;
        Ok(())
    }

    pub fn get_effect_report(&self, effect_block_index: u8) -> Result<SetEffectReport, ()> {
        // Check that the effect exists
        self.effects
            .get(effect_block_index as usize - 1)
            .ok_or(())?
            .ok_or(())?;

        // Read the Set Effect Report
        self.read_report(effect_address(effect_block_index), effect_block_index)
            //.map_err(|_| UsbError::ParseError)
    }

    pub fn get_type_specific_block_offsets(
        &self,
        effect_block_index: u8,
    ) -> Result<[usize; 2], ()> {
        let set_effect_report = self.get_effect_report(effect_block_index)?;

        Ok([
            set_effect_report.type_specific_block_offset_instance_1 as usize,
            set_effect_report.type_specific_block_offset_instance_2 as usize,
        ])
    }

    pub fn available(&self) -> usize {
        self.buffer.len() - self.allocated
    }

    pub fn pool_size(&self) -> usize {
        self.buffer.len()
    }
}

pub fn effect_address(effect_block_index: u8) -> usize {
    (effect_block_index - 1) as usize * SetEffectReport::RAM_SIZE
}

