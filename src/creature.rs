use bevy::math::primitives::{Circle, Rectangle};
use bevy::prelude::*;
use std::time::Duration;

use crate::limb::Limb;
use crate::oscillator::Oscillator;

const NUM_ARMS: usize = 16;
const SEGMENTS_PER_ARM: usize = 16;

const SEG_LENGTH: f32 = 30.0;
const SEG_THICKNESS: f32 = 10.0;

const BODY_RADIUS: f32 = 35.0;

const WAVE_FREQUENCY_HZ: f32 = 0.4;
const BASE_AMPLITUDE_RAD: f32 = 0.2;

#[derive(Component)]
#[require(Transform, Visibility, Oscillator, Children)]
pub struct Creature;

pub fn spawn_creature(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Shared meshes/materials
    let limb_mesh = meshes.add(Rectangle::new(SEG_LENGTH, SEG_THICKNESS));
    let limb_mat = materials.add(Color::srgb(0.6, 0.1, 0.8));
    let body_mesh = meshes.add(Circle::new(BODY_RADIUS));
    let body_mat = materials.add(Color::srgb(0.3, 0.05, 0.4));

    // Single oscillator on the creature that outputs one angle for all joints.
    let omega = std::f32::consts::TAU * WAVE_FREQUENCY_HZ;
    let root = commands
        .spawn((
            Creature,
            Oscillator {
                function: Box::new(move |elapsed: Duration| {
                    let t = elapsed.as_secs_f32();
                    BASE_AMPLITUDE_RAD * (omega * t).sin()
                }),
            },
            Name::new("Creature"),
        ))
        .id();

    // Visual body
    commands.entity(root).with_children(|p| {
        p.spawn((
            Mesh2d(body_mesh.clone()),
            MeshMaterial2d(body_mat.clone()),
            Transform::from_translation(Vec3::new(0.0, 0.0, -0.1)),
            Name::new("Body"),
        ));
    });

    // Arms
    for arm in 0..NUM_ARMS {
        let base_angle = (arm as f32 / NUM_ARMS as f32) * std::f32::consts::TAU;

        // Base joint, oriented outward (static)
        let base_joint = commands
            .spawn((
                Transform::from_rotation(Quat::from_rotation_z(base_angle)),
                Name::new(format!("Arm {arm} BaseJoint")),
            ))
            .id();

        commands.entity(root).add_children(&[base_joint]);

        let mut current_joint = base_joint;

        for seg in 0..SEGMENTS_PER_ARM {
            // Each limb joint gets the same local rotation from the creature.
            let seg_joint = commands
                .spawn((
                    Limb,
                    Transform::default(),
                    Name::new(format!("Arm {arm} Joint {seg}")),
                ))
                .id();

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

            // Next joint at the tip of this segment
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
