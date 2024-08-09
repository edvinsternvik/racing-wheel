use fixed_num::{fractional::Frac, Fixed16, Frac16};
use force_feedback::{
    effect::{Effect, EffectParameter::*},
    ffb::calculate_force_feedback,
    reports::{EffectType, SetConditionReport, SetEffectReport},
};
use std::{
    io::{stdout, Write},
    iter::repeat,
    thread::sleep,
    time::Duration,
};

fn get_bar<const N: u64>(value: Fixed16<N>, bar_len: i16) -> String {
    let value_bar = value.to_frac(bar_len / 2).value() + 1 + bar_len / 2;
    let value_bar = i16::clamp(value_bar, 1, bar_len);
    let mut bar_str = repeat('-').take(bar_len as usize + 1).collect::<Vec<_>>();
    bar_str[bar_len as usize / 2] = '|';
    bar_str[value_bar as usize - 1] = '█';

    bar_str.into_iter().collect()
}

fn main() {
    let effect = Effect {
        effect_report: Some(SetEffectReport {
            effect_block_index: 0,
            effect_type: EffectType::Spring,
            duration: Some(10_000),
            trigger_repeat_interval: 0,
            sample_period: None,
            gain: Frac::new(1, 1).convert(),
            trigger_button: 0,
            axis_x_enable: true,
            axis_y_enable: true,
            direction_enable: true,
            direction_instance_1: 0,
            direction_instance_2: 0,
            start_delay: 0,
            type_specific_block_offset_instance_1: 0,
            type_specific_block_offset_instance_2: 0,
        }),
        parameter_1: Some(Condition(SetConditionReport {
            effect_block_index: 0,
            parameter_block_offset: 0,
            type_specific_block_offset_instance_1: 0,
            type_specific_block_offset_instance_2: 0,
            cp_offset: 0.into(),
            positive_coefficient: Frac::new(-1, 1).convert(),
            negative_coefficient: Frac::new(-1, 1).convert(),
            positive_saturation: Frac::new(1, 1).convert(),
            negative_saturation: Frac::new(1, 1).convert(),
            dead_band: Frac::new(0, 1).convert(),
        })),
        parameter_2: Some(Condition(SetConditionReport {
            effect_block_index: 0,
            parameter_block_offset: 0,
            type_specific_block_offset_instance_1: 0,
            type_specific_block_offset_instance_2: 0,
            cp_offset: 0.into(),
            positive_coefficient: 0.into(),
            negative_coefficient: 0.into(),
            positive_saturation: 0.into(),
            negative_saturation: 0.into(),
            dead_band: 0.into(),
        })),
    };
    let mut time = 0;
    let mut prev_pos = 0.into();
    let mut prev_vel = 0.into();
    let mut prev_acc = 0.into();
    let dt = 10;
    let d_smooth = Frac16::new(1, 10);
    let d_smooth_inv = Frac16::new(d_smooth.denom() - d_smooth.value(), d_smooth.denom());

    loop {
        let position = ((f32::sin(time as f32 / 1000.0) * 2400.0) as i16).into();
        let velocity =
            (position - prev_pos) * Frac16::new(1000, dt) * d_smooth + prev_vel * d_smooth_inv;
        let acceleration =
            (velocity - prev_vel) * Frac16::new(1000, dt) * d_smooth + prev_acc * d_smooth_inv;
        prev_pos = position;
        prev_vel = velocity;
        prev_acc = acceleration;

        let ffb = calculate_force_feedback(&effect, time, position, velocity, acceleration);

        print!(
            "\rX: |{}| dX: |{}| ddX: |{}| FFB: |{}|",
            get_bar(position, 20),
            get_bar(velocity, 20),
            get_bar(acceleration, 20),
            get_bar(ffb, 80)
        );
        let _ = stdout().flush();

        sleep(Duration::from_millis(dt as u64));
        time += dt as u32;
    }
}
