use crate::{
    effect::{Effect, EffectParameter},
    reports::{
        EffectType, SetCondition, SetConstantForce,
        SetEffect, SetEnvelope, SetPeriodic, SetRampForce,
    },
};

pub fn calculate_force_feedback(
    effect: &Effect,
    time: u32,
    position: f32,
    velocity: f32,
    acceleration: f32,
) -> f32 {
    use EffectParameter::*;

    if let Some(duration) = effect.effect_report.map(|e| e.duration).flatten() {
        if time > duration as u32 {
            return 0.0;
        }
    }

    match (effect.effect_report, effect.parameter_1, effect.parameter_2) {
        (Some(e), Some(ConstantForce(p1)), None) => constant_ffb(&e, &p1, None, time),
        (Some(e), Some(ConstantForce(p1)), Some(Envelope(p2))) => constant_ffb(&e, &p1, Some(&p2), time),
        (Some(e), Some(RampForce(p1)), None) => ramp_ffb(&e, &p1, None, time),
        (Some(e), Some(RampForce(p1)), Some(Envelope(p2))) => ramp_ffb(&e, &p1, Some(&p2), time),
        (Some(_), Some(CustomForce(_p)), None) => 0.0,
        (Some(e), Some(Periodic(p1)), None) => periodic_ffb(&e, &p1, None, time),
        (Some(e), Some(Periodic(p1)), Some(Envelope(p2))) => periodic_ffb(&e, &p1, Some(&p2), time),
        (Some(e), Some(Condition(p1)), _) => match e.effect_type {
            EffectType::Spring => condition_ffb(&e, &p1, position),
            EffectType::Damper => condition_ffb(&e, &p1, velocity),
            EffectType::Inertia => condition_ffb(&e, &p1, acceleration),
            EffectType::Friction => 0.0,
            _ => 0.0,
        },
        _ => 0.0,
    }
}

fn calculate_envelope(envelope: Option<&SetEnvelope>, time: u32, duration: Option<u16>) -> f32 {
    if let Some(envelope) = envelope {
        let mut result = 1.0;
        if time < envelope.attack_time {
            let fade_force = envelope.attack_level
                + (1.0 - envelope.attack_level) * (time as f32 / envelope.attack_time as f32);
            result = f32::min(result, fade_force);
        }
        if let Some(duration) = duration {
            let duration = duration as u32;

            if time <= duration && time + envelope.fade_time > duration {
                let fade_force = envelope.fade_level
                    + (1.0 - envelope.fade_level)
                        * ((duration - time) as f32 / envelope.fade_time as f32);

                result = f32::min(result, fade_force);
            }
        }

        result
    } else {
        1.0
    }
}

fn condition_force(metric: f32, condition: &SetCondition) -> f32 {
    let force = if metric < condition.cp_offset - condition.dead_band {
        let velocity_delta = metric - (condition.cp_offset - condition.dead_band);
        condition.negative_coefficient * velocity_delta
    } else if metric > condition.cp_offset + condition.dead_band {
        let velocity_delta = metric - (condition.cp_offset + condition.dead_band);
        condition.positive_coefficient * velocity_delta
    } else {
        0.0
    };

    f32::clamp(
        force,
        -condition.negative_saturation,
        condition.positive_saturation,
    )
}

fn constant_ffb(
    effect: &SetEffect,
    constant_force: &SetConstantForce,
    envelope: Option<&SetEnvelope>,
    time: u32,
) -> f32 {
    let force = constant_force.magnitude;
    let envelope = calculate_envelope(envelope, time, effect.duration);
    force * envelope * effect.gain
}

fn ramp_ffb(
    effect: &SetEffect,
    ramp_force: &SetRampForce,
    envelope: Option<&SetEnvelope>,
    time: u32,
) -> f32 {
    if let Some(duration) = effect.duration {
        let force = ramp_force.ramp_start
            + (ramp_force.ramp_end - ramp_force.ramp_start) * (time as f32 / duration as f32);

        let envelope = calculate_envelope(envelope, time, effect.duration);
        force * envelope * effect.gain
    } else {
        0.0
    }
}

fn condition_ffb(
    effect: &SetEffect,
    condition_1: &SetCondition,
    metric: f32,
) -> f32 {
    let force = condition_force(metric, condition_1);
    force * effect.gain
}

fn periodic_ffb(
    effect: &SetEffect,
    periodic: &SetPeriodic,
    envelope: Option<&SetEnvelope>,
    time: u32,
) -> f32 {
    let f = match effect.effect_type{
        EffectType::Square => square_fn,
        EffectType::Sine => sine_fn,
        EffectType::Triangle => triangle_fn,
        EffectType::SawtoothUp => sawtooth_up_fn,
        EffectType::SawtoothDown => sawtooth_down_fn,
        _ => |_| 0.0,
    };
    let effect_time = time + ((periodic.phase as u64 * periodic.period as u64) / 36_000) as u32;

    let force_norm = f((effect_time % periodic.period) as f32 / periodic.period as f32);
    let force = periodic.magnitude * force_norm;

    let envelope = calculate_envelope(envelope, time, effect.duration);

    force * envelope * effect.gain
}

fn square_fn(time: f32) -> f32 {
    let t = time - (time as i64) as f32;
    if t >= 0.5 { 1.0 } else { -1.0 }
}

fn sine_fn(time: f32) -> f32 {
    const LUT_SAMPLES: usize = 64;
    const SIN_LUT: [i16; LUT_SAMPLES + 1] = [
        0, 804, 1607, 2410, 3211, 4011, 4807, 5601, 6392, 7179, 7961, 8739, 9511, 10278, 11038,
        11792, 12539, 13278, 14009, 14732, 15446, 16150, 16845, 17530, 18204, 18867, 19519, 20159,
        20787, 21402, 22004, 22594, 23169, 23731, 24278, 24811, 25329, 25831, 26318, 26789, 27244,
        27683, 28105, 28510, 28897, 29268, 29621, 29955, 30272, 30571, 30851, 31113, 31356, 31580,
        31785, 31970, 32137, 32284, 32412, 32520, 32609, 32678, 32727, 32757, 32767,
    ];

    let force_i16 = match (time * 4.0) as u8 {
        0 => SIN_LUT[((time - 0.0) * 4.0 * LUT_SAMPLES as f32) as usize],
        1 => SIN_LUT[((0.5 - time) * 4.0 * LUT_SAMPLES as f32) as usize],
        2 => -SIN_LUT[((time - 0.5) * 4.0 * LUT_SAMPLES as f32) as usize],
        _ => -SIN_LUT[((1.0 - time) * 4.0 * LUT_SAMPLES as f32) as usize],
    };

    force_i16 as f32 / i16::MAX as f32
}

fn triangle_fn(time: f32) -> f32 {
    2.0 * if time < 0.5 { time } else { 1.0 - time }
}

fn sawtooth_up_fn(time: f32) -> f32 {
    2.0 * if time < 0.5 { time } else { time - 1.0 }
}

fn sawtooth_down_fn(time: f32) -> f32 {
    -sawtooth_up_fn(time)
}
