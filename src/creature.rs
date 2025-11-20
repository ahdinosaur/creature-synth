use bevy::prelude::*;

use crate::limb::{Limb, LimbAssetStore, LimbPlan, LimbSegmentTypeId};
use crate::oscillator::{Oscillator, Wave};

#[derive(Component)]
#[require(Transform, Visibility, Children)]
pub struct Creature;

#[derive(Component)]
#[require(Transform, Visibility)]
pub struct CreatureBody;

const BODY_RADIUS: f32 = 35.0;

/// A creature plan is a list of limbs.
#[derive(Debug, Clone)]
pub struct CreaturePlan {
    pub limbs: Vec<LimbPlan>,
}

/// A collection of creatures to spawn, with a transform applied to the grouparent.
#[derive(Resource, Debug, Clone)]
pub struct CreaturesPlan {
    pub creatures: Vec<CreaturePlan>,
    pub transform: Transform,
}

/// Build an example plan:
/// - 6 creatures
/// - each with 8 limbs
/// - each limb has 16 segments
/// - all limbs run the same sine oscillator (amplitude 0.2, frequency 0.4)
/// - segments alternate Rectangle and Disk types along the limb
pub fn example_creatures_plan() -> CreaturesPlan {
    let limb_count = 8;
    let segment_count = 16;

    let oscillator = Oscillator::new(Wave::Sine, 0.2, 0.4);

    let segments: Vec<LimbSegmentTypeId> = (0..segment_count)
        .map(|i| {
            if i % 2 == 0 {
                LimbSegmentTypeId::Rectangle
            } else {
                LimbSegmentTypeId::Disk
            }
        })
        .collect();

    let limb = LimbPlan {
        oscillator: oscillator.clone(),
        segments: segments.clone(),
    };

    let creature = CreaturePlan {
        limbs: std::iter::repeat_n(limb, limb_count).collect(),
    };

    let creatures: Vec<CreaturePlan> = std::iter::repeat_n(creature, 6).collect();

    CreaturesPlan {
        creatures,
        transform: Transform::default(),
    }
}

/// Spawn all creatures described by the CreaturesPlan resource.
/// Each creature's body type is inferred from its first limb's first segment type.
pub fn spawn_creatures(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut store: ResMut<LimbAssetStore>,
    plans: Res<CreaturesPlan>,
) {
    // Root for all creatures, so the group transform can be applied once.
    let root = commands
        .spawn((Name::new("Creatures"), plans.transform, Visibility::Visible))
        .id();

    let body_mesh = meshes.add(Circle::new(BODY_RADIUS));
    let body_mat = materials.add(Color::srgb(0.3, 0.05, 0.4));

    for (creature_i, creature_plan) in plans.creatures.iter().enumerate() {
        // Create the creature root entity.
        let creature = commands
            .spawn((Creature, Name::new(format!("Creature {creature_i}"))))
            .id();

        // Attach creature under the global root so the group transform applies.
        commands.entity(root).add_children(&[creature]);

        // Visual body
        commands.entity(creature).with_children(|parent| {
            parent.spawn((
                CreatureBody,
                Name::new("Body"),
                Mesh2d(body_mesh.clone()),
                MeshMaterial2d(body_mat.clone()),
                Transform::from_translation(Vec3::new(0.0, 0.0, -0.1)),
            ));
        });

        // Limbs for this creature (distributed evenly around a circle).
        let limb_count = creature_plan.limbs.len().max(1);
        for (limb_index, limb_plan) in creature_plan.limbs.iter().enumerate() {
            let angle = std::f32::consts::TAU * limb_index as f32 / limb_count as f32;
            let limb_oscillator: Oscillator = limb_plan.oscillator.clone();

            let limb = commands
                .spawn((
                    Limb,
                    limb_oscillator,
                    Name::new(format!("Limb {limb_index}")),
                    Transform::from_rotation(Quat::from_rotation_z(angle)),
                ))
                .id();

            commands.entity(creature).add_children(&[limb]);

            // Build the chain of segments for this limb.
            let mut current_parent = limb;
            for (segment_index, type_id) in limb_plan.segments.iter().copied().enumerate() {
                // Ensure assets for this segment type exist.
                type_id.ensure_assets(&mut store, &mut meshes, &mut materials);

                // Spawn the segment and get the outgoing joint to chain the next one.
                let next_joint = type_id.spawn_segment(
                    &mut commands,
                    current_parent,
                    limb_index,
                    segment_index,
                    &store,
                );
                current_parent = next_joint;
            }
        }
    }
}
