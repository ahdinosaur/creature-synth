use bevy::prelude::*;

use crate::oscillator::Oscillator;

#[derive(Component)]
#[require(Oscillator, Transform, Visibility, Children)]
pub struct Limb;

#[derive(Component)]
#[require(Transform, Visibility, Children)]
pub struct LimbSegment;

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
    mut limb_segments: Query<&mut Transform, With<LimbSegment>>,
) {
    for (osc, limb_entity) in &limbs {
        let angle = osc.sample(time.elapsed());

        for child in children.iter_descendants(limb_entity) {
            if let Ok(mut transform) = limb_segments.get_mut(child) {
                transform.rotation = Quat::from_rotation_z(angle);
            }
        }
    }
}
