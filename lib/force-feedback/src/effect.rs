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

// Helper functions
pub fn create_spring_effect(
    gain: f32,
    duration: Option<u16>,
    cp_offset: f32,
    positive_coefficient: f32,
    negative_coefficient: f32,
    positive_saturation: f32,
    negative_saturation: f32,
    dead_band: f32,
) -> Effect {
    Effect {
        effect_report: Some(SetEffect {
            effect_type: EffectType::Spring,
            duration,
            gain,
            ..Default::default()
        }),
        parameter_1: Some(EffectParameter::Condition(SetCondition {
            cp_offset,
            positive_coefficient,
            negative_coefficient,
            positive_saturation,
            negative_saturation,
            dead_band,
            ..Default::default()
        })),
        parameter_2: Some(EffectParameter::Condition(SetCondition {
            cp_offset,
            positive_coefficient,
            negative_coefficient,
            positive_saturation,
            negative_saturation,
            dead_band,
            ..Default::default()
        })),
    }
}
