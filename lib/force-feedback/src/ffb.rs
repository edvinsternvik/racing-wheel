use crate::{
    effect::{Effect, EffectParameter},
    reports::{
        EffectType, FixedFFB, FixedSteering, SetConditionReport, SetConstantForceReport,
        SetEffectReport, SetEnvelopeReport, SetPeriodicReport, SetRampForceReport,
    },
};
use fixed_num::{Frac16, FracU32};

pub fn calculate_force_feedback(
    effect: &Effect,
    time: u32,
    position: FixedSteering,
    velocity: FixedSteering,
    acceleration: FixedSteering,
) -> FixedFFB {
    use EffectParameter::*;

    if let Some(duration) = effect.effect_report.map(|e| e.duration).flatten() {
        if time > duration as u32 {
            return 0.into();
        }
    }

    match (effect.effect_report, effect.parameter_1, effect.parameter_2) {
        (Some(e), Some(ConstantForce(p1)), Some(Envelope(p2))) => constant_ffb(&e, &p1, &p2, time),
        (Some(e), Some(RampForce(p1)), Some(Envelope(p2))) => ramp_ffb(&e, &p1, &p2, time),
        (Some(_), Some(CustomForce(_p)), None) => 0.into(),
        (Some(e), Some(Periodic(p1)), Some(Envelope(p2))) => match e.effect_type {
            EffectType::Square => periodic_ffb(&e, &p1, &p2, time, square_fn),
            EffectType::Sine => periodic_ffb(&e, &p1, &p2, time, sine_fn),
            EffectType::Triangle => periodic_ffb(&e, &p1, &p2, time, triangle_fn),
            EffectType::SawtoothUp => periodic_ffb(&e, &p1, &p2, time, sawtooth_up_fn),
            EffectType::SawtoothDown => periodic_ffb(&e, &p1, &p2, time, sawtooth_down_fn),
            _ => 0.into(),
        },
        (Some(e), Some(Condition(p1)), Some(Condition(p2))) => match e.effect_type {
            EffectType::Spring => condition_ffb(&e, &p1, &p2, position),
            EffectType::Damper => condition_ffb(&e, &p1, &p2, velocity),
            EffectType::Inertia => condition_ffb(&e, &p1, &p2, acceleration),
            EffectType::Friction => 0.into(),
            _ => 0.into(),
        },
        _ => 0.into(),
    }
}

fn calculate_envelope(envelope: &SetEnvelopeReport, time: u32, duration: Option<u16>) -> FixedFFB {
    let mut result = FixedFFB::one();
    if time < envelope.attack_time {
        let fade_force = envelope.attack_level
            + (FixedFFB::one() - envelope.attack_level) * FracU32::new(time, envelope.attack_time);
        result = FixedFFB::min(result, fade_force);
    }
    if let Some(duration) = duration {
        let duration = duration as u32;

        if time <= duration && time + envelope.fade_time > duration {
            let fade_force = envelope.fade_level
                + (FixedFFB::one() - envelope.fade_level)
                    * FracU32::new(duration - time, envelope.fade_time);

            result = FixedFFB::min(result, fade_force);
        }
    }

    result
}

fn condition_force(metric: FixedSteering, condition: &SetConditionReport) -> FixedFFB {
    let metric = metric.convert();
    let force = if metric < condition.cp_offset - condition.dead_band {
        let velocity_delta = metric - (condition.cp_offset - condition.dead_band);
        condition.negative_coefficient * velocity_delta
    } else if metric > condition.cp_offset + condition.dead_band {
        let velocity_delta = metric - (condition.cp_offset + condition.dead_band);
        condition.positive_coefficient * velocity_delta
    } else {
        0.into()
    };

    FixedFFB::clamp(
        force,
        -condition.negative_saturation,
        condition.positive_saturation,
    )
}

fn constant_ffb(
    effect: &SetEffectReport,
    constant_force: &SetConstantForceReport,
    envelope: &SetEnvelopeReport,
    time: u32,
) -> FixedFFB {
    let force = constant_force.magnitude;
    let envelope = calculate_envelope(envelope, time, effect.duration);
    force * envelope * effect.gain
}

fn ramp_ffb(
    effect: &SetEffectReport,
    ramp_force: &SetRampForceReport,
    envelope: &SetEnvelopeReport,
    time: u32,
) -> FixedFFB {
    if let Some(duration) = effect.duration {
        let force = ramp_force.ramp_start
            + (ramp_force.ramp_end - ramp_force.ramp_start) * FracU32::new(time, duration as u32);

        let envelope = calculate_envelope(envelope, time, effect.duration);
        force * envelope * effect.gain
    } else {
        0.into()
    }
}

fn condition_ffb(
    effect: &SetEffectReport,
    condition_1: &SetConditionReport,
    _condition_2: &SetConditionReport,
    metric: FixedSteering,
) -> FixedFFB {
    let force = condition_force(metric, condition_1);
    force * effect.gain
}

fn periodic_ffb(
    effect: &SetEffectReport,
    periodic: &SetPeriodicReport,
    envelope: &SetEnvelopeReport,
    time: u32,
    f: fn(FracU32) -> Frac16,
) -> FixedFFB {
    let effect_time = time + ((periodic.phase as u64 * periodic.period as u64) / 36_000) as u32;

    let force_norm = f(FracU32::new(effect_time, periodic.period));
    let force = periodic.magnitude * force_norm;

    let envelope = calculate_envelope(envelope, time, effect.duration);

    force * envelope * effect.gain
}

fn square_fn(time: FracU32) -> Frac16 {
    let t = time.value() % time.denom();
    let period_h = time.denom() / 2;
    let r = if t >= period_h { 1 } else { -1 };
    Frac16::new(r, 1)
}

fn sine_fn(time: FracU32) -> Frac16 {
    const LUT_SAMPLES: usize = 64;
    const SIN_LUT: [i16; LUT_SAMPLES + 1] = [
        0, 804, 1607, 2410, 3211, 4011, 4807, 5601, 6392, 7179, 7961, 8739, 9511, 10278, 11038,
        11792, 12539, 13278, 14009, 14732, 15446, 16150, 16845, 17530, 18204, 18867, 19519, 20159,
        20787, 21402, 22004, 22594, 23169, 23731, 24278, 24811, 25329, 25831, 26318, 26789, 27244,
        27683, 28105, 28510, 28897, 29268, 29621, 29955, 30272, 30571, 30851, 31113, 31356, 31580,
        31785, 31970, 32137, 32284, 32412, 32520, 32609, 32678, 32727, 32757, 32767,
    ];
    let period = time.denom() as u64;
    let mut t = (time.value() as u64 % period) * 4;
    let mut sign = 1;
    if t >= 2 * period {
        sign = -1;
        t -= 2 * period;
    }
    if t >= period {
        t = 2 * period - t;
    }
    let index = (t as u64 * LUT_SAMPLES as u64) / period as u64;
    let force = sign * SIN_LUT[index as usize];

    Frac16::new(force, i16::MAX)
}

fn triangle_fn(time: FracU32) -> Frac16 {
    let period = time.denom() as i64;
    let t = (time.value() as i64 % period) * 2;
    let t = if t < period { t } else { 2 * period - t };
    Frac16::new((2 * t - period) as i16, period as i16)
}

fn sawtooth_up_fn(time: FracU32) -> Frac16 {
    let period = time.denom() as i64;
    let t = time.value() as i64 % period;
    Frac16::new((2 * t - period) as i16, period as i16)
}

fn sawtooth_down_fn(time: FracU32) -> Frac16 {
    -sawtooth_up_fn(time)
}
