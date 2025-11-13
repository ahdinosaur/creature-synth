use bevy::prelude::*;
use std::time::Duration;

#[derive(Component)]
pub struct Oscillator {
    pub function: Box<dyn Fn(Duration) -> f32 + Send + Sync + 'static>,
}

impl Default for Oscillator {
    fn default() -> Self {
        let function = Box::new(|_time| 0_f32);
        Self { function }
    }
}
