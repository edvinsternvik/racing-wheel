use super::hid_reports::Report;
use core::mem::{size_of, size_of_val};
use force_feedback::{effect::Effect, reports::SetEffect};
use usb_hid_device::hid_device::HIDReportRAM;

pub struct RAMPool<const MAX_EFFECTS: usize, const CUSTOM_DATA_BUFFER_SIZE: usize> {
    custom_data_buffer: [u8; CUSTOM_DATA_BUFFER_SIZE],
    allocated: usize,
    effects: [Option<Effect>; MAX_EFFECTS],
}

impl<const MAX_EFFECTS: usize, const CUSTOM_DATA_BUFFER_SIZE: usize>
    RAMPool<MAX_EFFECTS, CUSTOM_DATA_BUFFER_SIZE>
{
    pub fn new() -> Self {
        Self {
            custom_data_buffer: [0; CUSTOM_DATA_BUFFER_SIZE],
            allocated: MAX_EFFECTS * Report::<SetEffect>::RAM_SIZE,
            effects: [None; MAX_EFFECTS],
        }
    }

    pub fn get_effect_mut(&mut self, effect_block_index: u8) -> Option<&mut Effect> {
        self.effects
            .get_mut(effect_block_index as usize - 1)?
            .as_mut()
    }

    pub fn get_effect(&self, effect_block_index: u8) -> Option<&Effect> {
        self.effects.get(effect_block_index as usize - 1)?.as_ref()
    }

    pub fn new_effect(&mut self) -> Option<u8> {
        let effects = self.effects.iter().enumerate();
        let index = effects.filter(|e| e.1.is_none()).next().map(|e| e.0)?;
        *self.effects.get_mut(index)? = Some(Effect::default());
        Some(index as u8 + 1)
    }

    pub fn free_effect(&mut self, effect_block_index: u8) -> Result<(), ()> {
        let effect = self
            .effects
            .get_mut((effect_block_index - 1) as usize)
            .ok_or(())?;
        *effect = None;
        Ok(())
    }

    pub fn available(&self) -> usize {
        let n_effects_available = self.effects.iter().filter(|e| e.is_none()).count();

        n_effects_available * size_of::<Option<Effect>>()
            + (self.custom_data_buffer.len() - self.allocated)
    }

    pub fn pool_size(&self) -> usize {
        size_of_val(&self.effects) + self.custom_data_buffer.len()
    }
}
