use bevy::prelude::*;

use crate::limb::{
    CreaturePlan, CreaturesPlan, Limb, LimbAssetStore, LimbSegmentTypeId, LimbSegmentTypeIdExt,
};
use crate::oscillator::Oscillator;

#[derive(Component)]
#[require(Transform, Visibility, Children)]
pub struct Creature;

#[derive(Component)]
#[require(Transform, Visibility)]
pub struct CreatureBody;

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
        .spawn((
            Name::new("Creatures"),
            plans.transform.clone(),
            Visibility::Visible,
        ))
        .id();

    for (creature_i, creature_plan) in plans.creatures.iter().enumerate() {
        // Pick a body type based on the first limb's first segment type.
        let body_type_id = infer_body_type_id(creature_plan);

        // Ensure body assets exist for the chosen type.
        body_type_id.ensure_assets(&mut store, &mut meshes, &mut materials);

        // Create the creature root entity.
        let creature = commands
            .spawn((Creature, Name::new(format!("Creature {creature_i}"))))
            .id();

        // Attach creature under the global root so the group transform applies.
        commands.entity(root).add_children(&[creature]);

        // Spawn the body as a child of the creature.
        commands.entity(creature).with_children(|p| {
            let body_entity = body_type_id.spawn_body(p, &store);
            p.entity(body_entity).insert(CreatureBody);
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

fn infer_body_type_id(plan: &CreaturePlan) -> LimbSegmentTypeId {
    plan.limbs
        .first()
        .and_then(|l| l.segments.first().copied())
        .expect("CreaturePlan must have at least one limb and one segment")
}
