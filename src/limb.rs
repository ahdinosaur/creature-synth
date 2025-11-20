use bevy::ecs::relationship::RelatedSpawnerCommands;
use bevy::math::primitives::{Circle, Rectangle};
use bevy::prelude::*;
use std::collections::HashMap;

use crate::oscillator::{Oscillator, Wave};

#[derive(Component)]
#[require(Oscillator, Transform, Visibility, Children)]
pub struct Limb;

#[derive(Component)]
#[require(Transform, Visibility, Children)]
pub struct LimbSegment {
    pub segment_index: usize,
    pub type_id: LimbSegmentTypeId,
}

#[derive(Component)]
#[require(Transform, Visibility)]
pub struct LimbSegmentBody;

#[derive(Component)]
#[require(Transform, Visibility, Children)]
pub struct LimbSegmentJoint;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LimbSegmentTypeId {
    Rectangle,
    Disk,
}

/// Mesh and material handles used by a given type id.
#[derive(Clone)]
pub struct TypeHandles {
    pub segment_mesh: Handle<Mesh>,
    pub segment_material: Handle<ColorMaterial>,
    pub body_mesh: Handle<Mesh>,
    pub body_material: Handle<ColorMaterial>,
}

/// Cache of handles for each segment type id.
#[derive(Resource, Default)]
pub struct LimbAssetStore {
    map: HashMap<LimbSegmentTypeId, TypeHandles>,
}

impl LimbAssetStore {
    pub fn has(&self, id: LimbSegmentTypeId) -> bool {
        self.map.contains_key(&id)
    }

    pub fn insert(&mut self, id: LimbSegmentTypeId, handles: TypeHandles) {
        self.map.insert(id, handles);
    }

    pub fn get(&self, id: LimbSegmentTypeId) -> &TypeHandles {
        self.map
            .get(&id)
            .expect("LimbAssetStore: type handles not initialized")
    }
}

/// Trait implemented by each static segment type. All methods are associated
/// functions (no self), so there is no runtime state inside the types.
pub trait LimbSegmentType {
    fn ensure_assets(
        store: &mut LimbAssetStore,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<ColorMaterial>,
    );

    fn spawn_body(
        parent: &mut RelatedSpawnerCommands<'_, ChildOf>,
        store: &LimbAssetStore,
    ) -> Entity;

    fn spawn_segment(
        commands: &mut Commands,
        parent: Entity,
        limb_index: usize,
        segment_index: usize,
        store: &LimbAssetStore,
    ) -> Entity;

    fn flex_for_segment(segment_index: usize) -> f32;
}

/// Rectangle segment implementation.
pub struct RectType;

impl RectType {
    const SEGMENT_LENGTH: f32 = 20.0;
    const SEGMENT_MARGIN: f32 = 5.0;
    const SEGMENT_THICKNESS: f32 = 10.0;

    const BODY_RADIUS: f32 = 35.0;
    const BODY_Z: f32 = -0.1;

    fn segment_color() -> Color {
        Color::srgb(0.6, 0.1, 0.8)
    }
    fn body_color() -> Color {
        Color::srgb(0.3, 0.05, 0.4)
    }
}

impl LimbSegmentType for RectType {
    fn ensure_assets(
        store: &mut LimbAssetStore,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<ColorMaterial>,
    ) {
        let id = LimbSegmentTypeId::Rectangle;
        if store.has(id) {
            return;
        }
        let segment_mesh = meshes.add(Rectangle::new(
            Self::SEGMENT_LENGTH,
            Self::SEGMENT_THICKNESS,
        ));
        let segment_material = materials.add(Self::segment_color());
        let body_mesh = meshes.add(Circle::new(Self::BODY_RADIUS));
        let body_material = materials.add(Self::body_color());

        store.insert(
            id,
            TypeHandles {
                segment_mesh,
                segment_material,
                body_mesh,
                body_material,
            },
        );
    }

    fn spawn_body(
        parent: &mut RelatedSpawnerCommands<'_, ChildOf>,
        store: &LimbAssetStore,
    ) -> Entity {
        let h = store.get(LimbSegmentTypeId::Rectangle);
        parent
            .spawn((
                Name::new("Body"),
                Mesh2d(h.body_mesh.clone()),
                MeshMaterial2d(h.body_material.clone()),
                Transform::from_translation(Vec3::new(0.0, 0.0, Self::BODY_Z)),
            ))
            .id()
    }

    fn spawn_segment(
        commands: &mut Commands,
        parent: Entity,
        limb_index: usize,
        segment_index: usize,
        store: &LimbAssetStore,
    ) -> Entity {
        let h = store.get(LimbSegmentTypeId::Rectangle);

        let mut joint_out: Option<Entity> = None;
        commands.entity(parent).with_children(|parent| {
            let segment = parent
                .spawn((
                    LimbSegment {
                        segment_index,
                        type_id: LimbSegmentTypeId::Rectangle,
                    },
                    Name::new(format!("Limb {limb_index} Segment {segment_index}")),
                    Transform::default(),
                ))
                .id();

            segment.with_children(|parent| {
                // The rectangle "bone".
                let body = parent.spawn((
                    LimbSegmentBody,
                    Name::new(format!("Limb {limb_index} Segment {segment_index} Body")),
                    Mesh2d(h.segment_mesh.clone()),
                    MeshMaterial2d(h.segment_material.clone()),
                    Transform::from_translation(Vec3::new(
                        Self::SEGMENT_LENGTH / 2.0 + Self::SEGMENT_MARGIN,
                        0.0,
                        0.0,
                    )),
                ));

                // Outgoing joint for the next segment.
                let joint = parent
                    .spawn((
                        LimbSegmentJoint,
                        Name::new(format!("Limb {limb_index} Segment {segment_index} Joint")),
                        Transform::from_translation(Vec3::new(
                            Self::SEGMENT_LENGTH + 2.0 * Self::SEGMENT_MARGIN,
                            0.0,
                            0.0,
                        )),
                    ))
                    .id();
                joint_out = Some(joint);
            });
        });

        joint_out.expect("joint should have been spawned")
    }

    fn flex_for_segment(segment_index: usize) -> f32 {
        let base = 1.1;
        let pow = 1.1;
        1.0 + (base - 1.0) * (segment_index as f32).powf(pow)
    }
}

/// Disk segment implementation (circular beads).
pub struct DiskType;

impl DiskType {
    const DIAMETER: f32 = 14.0;
    const MARGIN: f32 = 6.0;

    const BODY_RADIUS: f32 = 30.0;
    const BODY_Z: f32 = -0.1;

    fn segment_color() -> Color {
        Color::srgb(0.15, 0.8, 0.35)
    }
    fn body_color() -> Color {
        Color::srgb(0.07, 0.35, 0.18)
    }
}

impl LimbSegmentType for DiskType {
    fn ensure_assets(
        store: &mut LimbAssetStore,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<ColorMaterial>,
    ) {
        let id = LimbSegmentTypeId::Disk;
        if store.has(id) {
            return;
        }
        let r = Self::DIAMETER / 2.0;

        let segment_mesh = meshes.add(Circle::new(r));
        let segment_material = materials.add(Self::segment_color());
        let body_mesh = meshes.add(Circle::new(Self::BODY_RADIUS));
        let body_material = materials.add(Self::body_color());

        store.insert(
            id,
            TypeHandles {
                segment_mesh,
                segment_material,
                body_mesh,
                body_material,
            },
        );
    }

    fn spawn_body(
        parent: &mut RelatedSpawnerCommands<'_, ChildOf>,
        store: &LimbAssetStore,
    ) -> Entity {
        let h = store.get(LimbSegmentTypeId::Disk);
        parent
            .spawn((
                Name::new("Body"),
                Mesh2d(h.body_mesh.clone()),
                MeshMaterial2d(h.body_material.clone()),
                Transform::from_translation(Vec3::new(0.0, 0.0, Self::BODY_Z)),
            ))
            .id()
    }

    fn spawn_segment(
        commands: &mut Commands,
        parent: Entity,
        limb_index: usize,
        segment_index: usize,
        store: &LimbAssetStore,
    ) -> Entity {
        let h = store.get(LimbSegmentTypeId::Disk);

        let r = Self::DIAMETER / 2.0;
        let center = r + Self::MARGIN;
        let step = 2.0 * (r + Self::MARGIN);

        let mut joint_out: Option<Entity> = None;
        commands.entity(parent).with_children(|parent| {
            let segment = parent
                .spawn((
                    LimbSegment {
                        segment_index,
                        type_id: LimbSegmentTypeId::Disk,
                    },
                    Name::new(format!("Limb {limb_index} Segment {segment_index}")),
                    Transform::default(),
                ))
                .id();

            // The bead at the center.
            parent.entity(segment).with_children(|parent| {
                parent.spawn((
                    LimbSegmentBody,
                    Name::new(format!("Limb {limb_index} Segment {segment_index} Body")),
                    Mesh2d(h.segment_mesh.clone()),
                    MeshMaterial2d(h.segment_material.clone()),
                    Transform::from_translation(Vec3::new(center, 0.0, 0.0)),
                ));
            });

            // Outgoing joint for the next bead.
            parent.entity(segment).with_children(|parent| {
                let joint = parent
                    .spawn((
                        LimbSegmentJoint,
                        Name::new(format!("Limb {limb_index} Segment {segment_index} Joint")),
                        Transform::from_translation(Vec3::new(step, 0.0, 0.0)),
                    ))
                    .id();
                joint_out = Some(joint);
            });
        });

        joint_out.expect("joint should have been spawned")
    }

    fn flex_for_segment(segment_index: usize) -> f32 {
        let base = 1.05;
        let pow = 1.0;
        1.0 + (base - 1.0) * (segment_index as f32).powf(pow)
    }
}

impl LimbSegmentTypeId {
    pub fn ensure_assets(
        &self,
        store: &mut LimbAssetStore,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<ColorMaterial>,
    ) {
        match self {
            LimbSegmentTypeId::Rectangle => RectType::ensure_assets(store, meshes, materials),
            LimbSegmentTypeId::Disk => DiskType::ensure_assets(store, meshes, materials),
        }
    }

    pub fn spawn_body(
        &self,
        parent: &mut RelatedSpawnerCommands<'_, ChildOf>,
        store: &LimbAssetStore,
    ) -> Entity {
        match self {
            LimbSegmentTypeId::Rectangle => RectType::spawn_body(parent, store),
            LimbSegmentTypeId::Disk => DiskType::spawn_body(parent, store),
        }
    }

    pub fn spawn_segment(
        &self,
        commands: &mut Commands,
        parent: Entity,
        limb_index: usize,
        segment_index: usize,
        store: &LimbAssetStore,
    ) -> Entity {
        match self {
            LimbSegmentTypeId::Rectangle => {
                RectType::spawn_segment(commands, parent, limb_index, segment_index, store)
            }
            LimbSegmentTypeId::Disk => {
                DiskType::spawn_segment(commands, parent, limb_index, segment_index, store)
            }
        }
    }

    pub fn flex_for_segment(&self, segment_index: usize) -> f32 {
        match self {
            LimbSegmentTypeId::Rectangle => RectType::flex_for_segment(segment_index),
            LimbSegmentTypeId::Disk => DiskType::flex_for_segment(segment_index),
        }
    }
}

/// Animate all limb segments with their limb oscillator and type-specific flex.
pub fn animate_limb_segments(
    children: Query<&Children>,
    limbs: Query<(&Oscillator, Entity), With<Limb>>,
    mut limb_segments: Query<(&mut Transform, &LimbSegment), With<LimbSegment>>,
) {
    for (osc, limb_entity) in &limbs {
        let angle = osc.sample();
        for child in children.iter_descendants(limb_entity) {
            if let Ok((mut transform, limb_segment)) = limb_segments.get_mut(child) {
                let flex = limb_segment
                    .type_id
                    .flex_for_segment(limb_segment.segment_index);
                transform.rotation = Quat::from_rotation_z(angle * flex);
            }
        }
    }
}

/// A limb plan specifies its oscillator and the per-segment types.
#[derive(Debug, Clone)]
pub struct LimbPlan {
    pub oscillator: Oscillator,
    pub segments: Vec<LimbSegmentTypeId>,
}

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
        limbs: std::iter::repeat(limb).take(limb_count).collect(),
    };

    let creatures: Vec<CreaturePlan> = std::iter::repeat(creature).take(6).collect();

    CreaturesPlan {
        creatures,
        transform: Transform::default(),
    }
}
