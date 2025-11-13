mod creature;
mod limb;
mod oscillator;

use bevy::prelude::*;

use crate::{creature::spawn_creature, limb::animate_limbs_from_creature};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, (setup_camera, spawn_creature))
        .add_systems(Update, animate_limbs_from_creature)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}
