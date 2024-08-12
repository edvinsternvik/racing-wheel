use crate::reports::{
    EffectType, SetCondition, SetConstantForce, SetCustomForce, SetEffect, SetEnvelope,
    SetPeriodic, SetRampForce,
};

#[derive(Clone, Copy, Default)]
pub struct Effect {
    pub effect_report: Option<SetEffect>,
    pub parameter_1: Option<EffectParameter>,
    pub parameter_2: Option<EffectParameter>,
}

impl Effect {
    pub fn is_complete(&self) -> bool {
        if let Some(effect) = self.effect_report {
            return match effect.effect_type {
                EffectType::CustomForceData => self.parameter_1.is_some(),
                _ => self.parameter_1.is_some() && self.parameter_2.is_some(),
            };
        }
        false
    }
}

#[derive(Clone, Copy)]
pub enum EffectParameter {
    Envelope(SetEnvelope),
    Condition(SetCondition),
    Periodic(SetPeriodic),
    ConstantForce(SetConstantForce),
    RampForce(SetRampForce),
    CustomForce(SetCustomForce),
}
