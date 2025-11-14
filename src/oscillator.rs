use bevy::prelude::*;
use std::{f32::consts::TAU, ops::Neg, time::Duration};

#[derive(Component, Default)]
#[allow(dead_code)]
pub enum Oscillator {
    #[default]
    Flat,
    Sin {
        frequency: f32,
        amplitude: f32,
    },
    Square {
        frequency: f32,
        amplitude: f32,
    },
    Triangle {
        frequency: f32,
        amplitude: f32,
    },
}

impl Oscillator {
    pub fn sample(&self, elapsed: Duration) -> f32 {
        let t = elapsed.as_secs_f32();
        match self {
            Oscillator::Flat => 0_f32,
            Oscillator::Sin {
                frequency,
                amplitude,
            } => {
                let w = TAU * frequency;
                amplitude * (w * t).sin()
            }
            Oscillator::Square {
                frequency,
                amplitude,
            } => {
                let w = TAU * frequency;
                let s = (w * t).sin();
                if s >= 0_f32 {
                    amplitude.to_owned()
                } else {
                    amplitude.neg()
                }
            }
            Oscillator::Triangle {
                frequency,
                amplitude,
            } => {
                let phase = (t * frequency + 0.25_f32) % 1_f32;
                let tri = 1_f32 - 4_f32 * (phase - 0.5_f32).abs();
                amplitude * tri
            }
        }
    }
}
