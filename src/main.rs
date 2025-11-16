mod creature;
mod limb;
mod oscillator;

use bevy::{dev_tools::fps_overlay::FpsOverlayPlugin, prelude::*};

use crate::{
    creature::spawn_creature,
    limb::animate_limb_segments,
    oscillator::{oscillator_tick, oscillator_user_update},
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(FpsOverlayPlugin {
            ..Default::default()
        })
        .add_systems(Startup, (setup_camera, spawn_creature))
        .add_systems(Update, oscillator_tick.before(animate_limb_segments))
        .add_systems(Update, oscillator_user_update)
        .add_systems(Update, animate_limb_segments)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}
