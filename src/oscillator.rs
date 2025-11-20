use bevy::prelude::*;
use std::{f32::consts::TAU, time::Duration};

/// Advance all oscillators by Time, with a capped delta.
pub fn oscillator_tick(time: Res<Time>, mut q: Query<&mut Oscillator>) {
    let dt = time.delta_secs().min(0.05);
    for mut osc in &mut q {
        osc.tick(dt);
    }
}

const FREQ_STEP: f32 = 0.05;
const MIN_FREQ: f32 = 0.0;
const MAX_FREQ: f32 = 2.0;

/// Simple user input: Up/Down arrow changes the target frequency of all
/// oscillators. Smoothing is handled by the oscillator itself.
pub fn oscillator_user_update(keys: Res<ButtonInput<KeyCode>>, mut q: Query<&mut Oscillator>) {
    let mut delta = 0.0;
    if keys.just_pressed(KeyCode::ArrowUp) {
        delta += FREQ_STEP;
    }
    if keys.just_pressed(KeyCode::ArrowDown) {
        delta -= FREQ_STEP;
    }
    if delta == 0.0 {
        return;
    }
    for mut osc in &mut q {
        let target = (osc.target_frequency() + delta).clamp(MIN_FREQ, MAX_FREQ);
        osc.set_frequency(target);
    }
}

#[derive(Debug, Clone, Copy, Default)]
#[allow(dead_code)]
pub enum Wave {
    #[default]
    Flat,
    Sine,
    Square,
    Triangle,
}

/// Frequency smoother with a time constant (tau).
/// - current: instantaneous frequency (Hz)
/// - target: desired frequency (Hz)
/// - tau: time constant; ~63% toward target in tau seconds
#[derive(Debug, Clone)]
pub struct Frequency {
    current: f32,
    target: f32,
    tau: f32,
}

impl Default for Frequency {
    fn default() -> Self {
        Self {
            current: 0.0,
            target: 0.0,
            tau: 0.03,
        }
    }
}

impl Frequency {
    pub fn new(hz: f32) -> Self {
        Self {
            current: hz,
            target: hz,
            ..Default::default()
        }
    }

    pub fn set_target(&mut self, hz: f32) {
        self.target = hz.max(0.0);
    }

    pub fn set_tau(&mut self, tau: Duration) {
        self.tau = tau.as_secs_f32().max(0.0);
    }

    pub fn target(&self) -> f32 {
        self.target
    }

    pub fn current(&self) -> f32 {
        self.current
    }

    // Exponential smoothing that is frame-rate independent.
    // alpha = 1 - exp(-dt / tau)
    pub fn update(&mut self, dt: f32) -> f32 {
        if dt <= 0.0 {
            return self.current;
        }
        if self.tau <= 0.0 {
            self.current = self.target;
            return self.current;
        }
        let alpha = 1.0 - (-dt / self.tau).exp();
        self.current += (self.target - self.current) * alpha;
        self.current
    }
}

#[derive(Component, Debug, Clone)]
pub struct Oscillator {
    wave: Wave,
    amplitude: f32,
    frequency: Frequency,
    phase: f32,
}

impl Default for Oscillator {
    fn default() -> Self {
        Self {
            wave: Wave::Sine,
            amplitude: 1.0,
            frequency: Frequency::default(),
            phase: 0.0,
        }
    }
}

impl Oscillator {
    pub fn new(wave: Wave, amplitude: f32, frequency: f32) -> Self {
        Self {
            wave,
            amplitude,
            frequency: Frequency::new(frequency),
            phase: 0.0,
        }
    }

    pub fn set_transition_time(&mut self, d: Duration) {
        self.frequency.set_tau(d);
    }

    // Set the target frequency. Glide is applied over the configured tau.
    pub fn set_frequency(&mut self, hz: f32) {
        self.frequency.set_target(hz);
    }

    pub fn target_frequency(&self) -> f32 {
        self.frequency.target()
    }

    pub fn current_frequency(&self) -> f32 {
        self.frequency.current()
    }

    // Advance the oscillator by dt using the average of f(t) and f(t+dt).
    pub fn tick(&mut self, dt: f32) {
        if dt <= 0.0 {
            return;
        }
        let f0 = self.frequency.current();
        let f1 = self.frequency.update(dt);
        let f_avg = 0.5 * (f0 + f1);
        self.phase = (self.phase + f_avg * dt).fract();
    }

    // Sample the current waveform at the stored phase.
    pub fn sample(&self) -> f32 {
        let a = self.amplitude;
        match self.wave {
            Wave::Flat => 0.0,
            Wave::Sine => {
                let phi = TAU * self.phase;
                a * phi.sin()
            }
            Wave::Square => {
                let phi = TAU * self.phase;
                if phi.sin() >= 0.0 {
                    a
                } else {
                    -a
                }
            }
            Wave::Triangle => {
                let p = (self.phase + 0.25).fract();
                let tri = 1.0 - 4.0 * (p - 0.5).abs();
                a * tri
            }
        }
    }
}
