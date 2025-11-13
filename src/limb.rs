use bevy::prelude::*;

use crate::creature::Creature;
use crate::oscillator::Oscillator;

#[derive(Component)]
#[require(Transform, Visibility, Children)]
pub struct Limb;

pub fn animate_limbs_from_creature(
    time: Res<Time>,
    creature_q: Query<&Oscillator, With<Creature>>,
    mut limbs_q: Query<&mut Transform, With<Limb>>,
) {
    let Ok(osc) = creature_q.single() else {
        return;
    };
    let angle = osc.sample(time.elapsed());

    for mut transform in &mut limbs_q {
        transform.rotation = Quat::from_rotation_z(angle);
    }
}
