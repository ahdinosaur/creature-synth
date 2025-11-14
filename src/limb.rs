use bevy::prelude::*;

use crate::oscillator::Oscillator;

#[derive(Component)]
#[require(Oscillator, Transform, Visibility, Children)]
pub struct Limb;

#[derive(Component)]
#[require(Transform, Visibility, Children)]
pub struct LimbSegment {
    pub segment_index: usize,
}

const SEGMENT_INDEX_FLEX_MULTIPLIER: f32 = 1.05;

#[derive(Component)]
#[require(Transform, Visibility)]
pub struct LimbSegmentBody;

#[derive(Component)]
#[require(Transform, Visibility, Children)]
pub struct LimbSegmentJoint;

pub fn animate_limb_segments(
    time: Res<Time>,
    children: Query<&Children>,
    limbs: Query<(&Oscillator, Entity), With<Limb>>,
    mut limb_segments: Query<(&mut Transform, &LimbSegment), With<LimbSegment>>,
) {
    for (osc, limb_entity) in &limbs {
        let angle = osc.sample(time.elapsed());

        for child in children.iter_descendants(limb_entity) {
            if let Ok((mut transform, limb_segment)) = limb_segments.get_mut(child) {
                let flex = 1.0
                    + (SEGMENT_INDEX_FLEX_MULTIPLIER - 1.0)
                        * (limb_segment.segment_index as f32).powf(1.1);
                transform.rotation = Quat::from_rotation_z(angle * flex);
            }
        }
    }
}
