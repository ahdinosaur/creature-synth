use std::time::Duration;

use bevy::math::primitives::{Circle, Rectangle};
use bevy::prelude::*;

const NUM_ARMS: usize = 8;
const SEGMENTS_PER_ARM: usize = 32;

const SEG_LENGTH: f32 = 10.0;
const SEG_THICKNESS: f32 = 10.0;

const BODY_RADIUS: f32 = 35.0;

// Oscillation params
const WAVE_FREQUENCY_HZ: f32 = 0.5;
const BASE_AMPLITUDE_RAD: f32 = 0.5; // ~28.6 degrees
const AMPLITUDE_DECAY: f32 = 1.;
const PHASE_PER_SEGMENT: f32 = 0.45; // radians

#[derive(Component)]
struct Oscillator {
    function: Box<dyn Fn(Duration) -> f32 + Send + Sync + 'static>,
}

impl Default for Oscillator {
    fn default() -> Self {
        let function = Box::new(|_time| 0_f32);
        Self { function }
    }
}

#[derive(Component)]
#[require(Transform, Visibility, Oscillator, Children)]
struct Creature;

#[derive(Component)]
#[require(Transform, Visibility, Oscillator, Children)]
struct Limb;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, (setup_camera, spawn_creature))
        .add_systems(Update, animate_oscillators)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn animate_oscillators(time: Res<Time>, mut q: Query<(&Oscillator, &mut Transform)>) {
    let t = time.elapsed();
    for (osc, mut transform) in &mut q {
        let angle = (osc.function)(t);
        transform.rotation = Quat::from_rotation_z(angle);
    }
}

fn spawn_creature(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Shared meshes/materials
    let limb_mesh = meshes.add(Rectangle::new(SEG_LENGTH, SEG_THICKNESS));
    let limb_mat = materials.add(Color::srgb(0.6, 0.1, 0.8));
    let body_mesh = meshes.add(Circle::new(BODY_RADIUS));
    let body_mat = materials.add(Color::srgb(0.3, 0.05, 0.4));

    let root = commands.spawn((Creature, Name::new("Creature"))).id();

    // Visual body
    commands.entity(root).with_children(|p| {
        p.spawn((
            Mesh2d(body_mesh.clone()),
            MeshMaterial2d(body_mat.clone()),
            Transform::from_translation(Vec3::new(0.0, 0.0, -0.1)),
            Name::new("Body"),
        ));
    });

    let omega = std::f32::consts::TAU * WAVE_FREQUENCY_HZ;

    for arm in 0..NUM_ARMS {
        let base_angle = (arm as f32 / NUM_ARMS as f32) * std::f32::consts::TAU;

        // Base joint, oriented outward (no Limb here; the chain below has 8 Limbs)
        let base_joint = commands
            .spawn((
                Transform::from_rotation(Quat::from_rotation_z(base_angle)),
                Name::new(format!("Arm {arm} BaseJoint")),
            ))
            .id();

        // Parent it to the creature
        commands.entity(root).add_children(&[base_joint]);

        // Build the chain: Joint_i (Limb+Osc) -> [mesh] -> NextJoint ...
        let mut current_joint = base_joint;

        for seg in 0..SEGMENTS_PER_ARM {
            // Oscillator for this segment (traveling wave + decay)
            let phase = base_angle + (seg as f32) * PHASE_PER_SEGMENT;
            let amplitude = BASE_AMPLITUDE_RAD * AMPLITUDE_DECAY.powi(seg as i32);

            let osc = Oscillator {
                function: Box::new(move |elapsed: Duration| {
                    let t = elapsed.as_secs_f32();
                    amplitude * (omega * t + phase)
                }),
            };

            // Segment pivot joint (this is "the limb" segment)
            let seg_joint = commands
                .spawn((Limb, osc, Name::new(format!("Arm {arm} Joint {seg}"))))
                .id();

            // Attach joint under current_joint
            commands.entity(current_joint).add_children(&[seg_joint]);

            // Visual rectangle, positioned so its left end is at the joint
            commands.entity(seg_joint).with_children(|p| {
                p.spawn((
                    Mesh2d(limb_mesh.clone()),
                    MeshMaterial2d(limb_mat.clone()),
                    Transform::from_translation(Vec3::new(SEG_LENGTH / 2.0, 0.0, 0.0)),
                    Name::new(format!("Arm {arm} Segment {seg}")),
                ));
            });

            // Next joint at the tip of this segment (no Limb here)
            let next_joint = commands
                .spawn((
                    Transform::from_translation(Vec3::new(SEG_LENGTH, 0.0, 0.0)),
                    Name::new(format!("Arm {arm} AfterJoint {seg}")),
                ))
                .id();

            commands.entity(seg_joint).add_children(&[next_joint]);
            current_joint = next_joint;
        }
    }
}
