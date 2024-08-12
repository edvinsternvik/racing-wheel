use force_feedback::{
    effect::{Effect, EffectParameter::*},
    ffb::calculate_force_feedback,
    reports::{EffectType, SetCondition, SetEffect},
};
use std::{
    io::{stdout, Write},
    iter::repeat,
    thread::sleep,
    time::Duration,
};

fn get_bar(value: f32, bar_len: i16) -> String {
    let value_bar = 1.0 + (value + 1.0) * bar_len as f32 / 2.0;
    let value_bar = i16::clamp(value_bar as i16, 1, bar_len);
    let mut bar_str = repeat('-').take(bar_len as usize + 1).collect::<Vec<_>>();
    bar_str[bar_len as usize / 2] = '|';
    bar_str[value_bar as usize - 1] = '█';

    bar_str.into_iter().collect()
}

fn main() {
    let effect = Effect {
        effect_report: Some(SetEffect {
            effect_block_index: 0,
            effect_type: EffectType::Spring,
            duration: Some(10_000),
            trigger_repeat_interval: 0,
            sample_period: None,
            gain: 1.0,
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
        parameter_1: Some(Condition(SetCondition {
            effect_block_index: 0,
            parameter_block_offset: 0,
            type_specific_block_offset_instance_1: 0,
            type_specific_block_offset_instance_2: 0,
            cp_offset: 0.0,
            positive_coefficient: -1.0,
            negative_coefficient: -1.0,
            positive_saturation: 1.0,
            negative_saturation: 1.0,
            dead_band: 0.0,
        })),
        parameter_2: Some(Condition(SetCondition {
            effect_block_index: 0,
            parameter_block_offset: 0,
            type_specific_block_offset_instance_1: 0,
            type_specific_block_offset_instance_2: 0,
            cp_offset: 0.0,
            positive_coefficient: 0.0,
            negative_coefficient: 0.0,
            positive_saturation: 0.0,
            negative_saturation: 0.0,
            dead_band: 0.0,
        })),
    };
    let mut time = 0;
    let mut prev_pos = 0.0;
    let mut prev_vel = 0.0;
    let mut prev_acc = 0.0;
    let dt = 10;
    let d_smooth = 0.1;
    let d_smooth_inv = 1.0 - d_smooth;

    loop {
        let position = f32::sin(time as f32 / 1000.0);
        let velocity =
            (position - prev_pos) * (1000.0 / dt as f32) * d_smooth + prev_vel * d_smooth_inv;
        let acceleration =
            (velocity - prev_vel) * (1000.0 / dt as f32) * d_smooth + prev_acc * d_smooth_inv;
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
