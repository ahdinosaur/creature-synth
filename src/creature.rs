use std::f32::consts::TAU;

use bevy::math::primitives::{Circle, Rectangle};
use bevy::prelude::*;

use crate::limb::{Limb, LimbSegment, LimbSegmentBody, LimbSegmentJoint};
use crate::oscillator::Oscillator;

const NUM_LIMBS: usize = 16;
const SEGMENTS_PER_LIMB: usize = 16;

const SEG_LENGTH: f32 = 20.0;
const SEG_MARGIN: f32 = 5.0;
const SEG_THICKNESS: f32 = 10.0;

const BODY_RADIUS: f32 = 35.0;

#[derive(Component)]
#[require(Transform, Visibility, Children)]
pub struct Creature;

#[derive(Component)]
#[require(Transform, Visibility)]
pub struct CreatureBody;

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

    let creature = commands.spawn((Creature, Name::new("Creature"))).id();

    // Visual body
    commands.entity(creature).with_children(|p| {
        p.spawn((
            CreatureBody,
            Name::new("Body"),
            Mesh2d(body_mesh.clone()),
            MeshMaterial2d(body_mat.clone()),
            Transform::from_translation(Vec3::new(0.0, 0.0, -0.1)),
        ));
    });

    // Arms
    for limb_index in 0..NUM_LIMBS {
        let limb_angle = (limb_index as f32 / NUM_LIMBS as f32) * TAU;
        let limb_osc = if limb_index % 2 == 0 {
            Oscillator::Sin {
                frequency: 0.4,
                amplitude: 0.2,
            }
        } else {
            Oscillator::Square {
                frequency: 0.4,
                amplitude: 0.2,
            }
        };

        let limb = commands
            .spawn((
                Limb,
                limb_osc,
                Name::new(format!("Limb {limb_index}")),
                Transform::from_rotation(Quat::from_rotation_z(limb_angle)),
            ))
            .id();

        commands.entity(creature).add_children(&[limb]);

        let mut current_limb_parent = limb;

        for segment_index in 0..SEGMENTS_PER_LIMB {
            // Each limb segment gets the same local rotation from the creature.
            let limb_segment = commands
                .spawn((
                    LimbSegment { segment_index },
                    Name::new(format!("Limb {limb_index} Segment {segment_index}")),
                    Transform::default(),
                ))
                .id();

            commands
                .entity(current_limb_parent)
                .add_children(&[limb_segment]);

            // Visual rectangle, positioned so its left end is at the joint
            commands.entity(limb_segment).with_children(|p| {
                p.spawn((
                    LimbSegmentBody,
                    Name::new(format!("Limb {limb_index} Segment {segment_index} Body")),
                    Mesh2d(limb_mesh.clone()),
                    MeshMaterial2d(limb_mat.clone()),
                    Transform::from_translation(Vec3::new(SEG_LENGTH / 2.0 + SEG_MARGIN, 0.0, 0.0)),
                ));
            });

            // Next joint at the tip of this segment
            let next_limb_joint = commands
                .spawn((
                    LimbSegmentJoint,
                    Name::new(format!("Limb {limb_index} Segment {segment_index} Joint")),
                    Transform::from_translation(Vec3::new(
                        SEG_LENGTH + 2_f32 * SEG_MARGIN,
                        0.0,
                        0.0,
                    )),
                ))
                .id();

            commands
                .entity(limb_segment)
                .add_children(&[next_limb_joint]);

            current_limb_parent = next_limb_joint;
        }
    }
}
