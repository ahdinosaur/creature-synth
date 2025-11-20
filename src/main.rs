mod creature;
mod limb;
mod oscillator;

use bevy::{dev_tools::fps_overlay::FpsOverlayPlugin, prelude::*};

use crate::{
    creature::{example_creatures_plan, spawn_creatures},
    limb::{animate_limb_segments, LimbAssetStore},
    oscillator::{oscillator_tick, oscillator_user_update},
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(FpsOverlayPlugin {
            ..Default::default()
        })
        // Resources: type asset cache and an example multi-creature plan.
        .insert_resource(LimbAssetStore::default())
        .insert_resource(example_creatures_plan())
        // Startup
        .add_systems(Startup, (setup_camera, spawn_creatures))
        // Oscillator updates
        .add_systems(Update, oscillator_tick.before(animate_limb_segments))
        .add_systems(Update, oscillator_user_update)
        // Animation
        .add_systems(Update, animate_limb_segments)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}
