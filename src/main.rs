use std::{sync::Arc, time::Duration};

use bevy::prelude::*;

fn main() {
    App::new().run();
}

#[derive(Component)]
struct Oscillator {
    function: Box<dyn Fn(Duration) -> f32 + Send + Sync + 'static>,
}

#[derive(Component)]
#[require(Transform, Children)]
struct Creature;

#[derive(Component)]
#[require(Transform, Children)]
struct Limb;
