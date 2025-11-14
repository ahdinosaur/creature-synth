use bevy::prelude::*;
use std::{f32::consts::TAU, ops::Neg, time::Duration};

#[derive(Component, Default)]
#[allow(dead_code)]
pub enum Wave {
    #[default]
    Flat,
    Sine,
    Square,
    Triangle,
}

#[derive(Component, Default)]
pub struct Oscillator {
    pub wave: Wave,
    pub frequency: f32,
    pub amplitude: f32,
}

impl Oscillator {
    pub fn sample(&self, elapsed: Duration) -> f32 {
        let Oscillator {
            wave,
            frequency,
            amplitude,
        } = self;
        let t = elapsed.as_secs_f32();
        match wave {
            Wave::Flat => 0_f32,
            Wave::Sine => {
                let w = TAU * frequency;
                amplitude * (w * t).sin()
            }
            Wave::Square => {
                let w = TAU * frequency;
                let s = (w * t).sin();
                if s >= 0_f32 {
                    amplitude.to_owned()
                } else {
                    amplitude.neg()
                }
            }
            Wave::Triangle => {
                let phase = (t * frequency + 0.25_f32) % 1_f32;
                let tri = 1_f32 - 4_f32 * (phase - 0.5_f32).abs();
                amplitude * tri
            }
        }
    }
}
