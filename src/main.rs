mod creature;
mod limb;
mod oscillator;

use bevy::prelude::*;

use crate::{creature::spawn_creature, limb::animate_limb_segments};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, (setup_camera, spawn_creature))
        .add_systems(Update, animate_limb_segments)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}
