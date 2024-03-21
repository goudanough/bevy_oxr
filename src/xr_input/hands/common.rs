use bevy::prelude::{
    Color, Commands, Component, Deref, DerefMut, Entity, Gizmos, Plugin, PostUpdate, Query,
    Resource, SpatialBundle, Startup, Transform,
};

use crate::xr_input::{trackers::OpenXRTracker, Hand};

use super::{BoneTrackingStatus, HandBone};

/// add debug renderer for controllers
#[derive(Default)]
pub struct OpenXrHandInput;

impl Plugin for OpenXrHandInput {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Startup, spawn_hand_entities);
    }
}

/// add debug renderer for controllers
#[derive(Default)]
pub struct HandInputDebugRenderer;

impl Plugin for HandInputDebugRenderer {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(PostUpdate, draw_hand_entities);
    }
}

#[derive(Resource, Default, Clone, Copy)]
pub struct HandsResource {
    pub left: HandResource,
    pub right: HandResource,
}
#[derive(Clone, Copy)]
pub struct HandResource {
    pub palm: Entity,
    pub wrist: Entity,
    pub thumb: ThumbResource,
    pub index: IndexResource,
    pub middle: MiddleResource,
    pub ring: RingResource,
    pub little: LittleResource,
}

impl Default for HandResource {
    fn default() -> Self {
        Self {
            palm: Entity::PLACEHOLDER,
            wrist: Entity::PLACEHOLDER,
            thumb: Default::default(),
            index: Default::default(),
            middle: Default::default(),
            ring: Default::default(),
            little: Default::default(),
        }
    }
}
#[derive(Clone, Copy)]
pub struct ThumbResource {
    pub metacarpal: Entity,
    pub proximal: Entity,
    pub distal: Entity,
    pub tip: Entity,
}

impl Default for ThumbResource {
    fn default() -> Self {
        Self {
            metacarpal: Entity::PLACEHOLDER,
            proximal: Entity::PLACEHOLDER,
            distal: Entity::PLACEHOLDER,
            tip: Entity::PLACEHOLDER,
        }
    }
}
#[derive(Clone, Copy)]
pub struct IndexResource {
    pub metacarpal: Entity,
    pub proximal: Entity,
    pub intermediate: Entity,
    pub distal: Entity,
    pub tip: Entity,
}

impl Default for IndexResource {
    fn default() -> Self {
        Self {
            metacarpal: Entity::PLACEHOLDER,
            proximal: Entity::PLACEHOLDER,
            intermediate: Entity::PLACEHOLDER,
            distal: Entity::PLACEHOLDER,
            tip: Entity::PLACEHOLDER,
        }
    }
}
#[derive(Clone, Copy)]
pub struct MiddleResource {
    pub metacarpal: Entity,
    pub proximal: Entity,
    pub intermediate: Entity,
    pub distal: Entity,
    pub tip: Entity,
}
impl Default for MiddleResource {
    fn default() -> Self {
        Self {
            metacarpal: Entity::PLACEHOLDER,
            proximal: Entity::PLACEHOLDER,
            intermediate: Entity::PLACEHOLDER,
            distal: Entity::PLACEHOLDER,
            tip: Entity::PLACEHOLDER,
        }
    }
}
#[derive(Clone, Copy)]
pub struct RingResource {
    pub metacarpal: Entity,
    pub proximal: Entity,
    pub intermediate: Entity,
    pub distal: Entity,
    pub tip: Entity,
}
impl Default for RingResource {
    fn default() -> Self {
        Self {
            metacarpal: Entity::PLACEHOLDER,
            proximal: Entity::PLACEHOLDER,
            intermediate: Entity::PLACEHOLDER,
            distal: Entity::PLACEHOLDER,
            tip: Entity::PLACEHOLDER,
        }
    }
}
#[derive(Clone, Copy)]
pub struct LittleResource {
    pub metacarpal: Entity,
    pub proximal: Entity,
    pub intermediate: Entity,
    pub distal: Entity,
    pub tip: Entity,
}
impl Default for LittleResource {
    fn default() -> Self {
        Self {
            metacarpal: Entity::PLACEHOLDER,
            proximal: Entity::PLACEHOLDER,
            intermediate: Entity::PLACEHOLDER,
            distal: Entity::PLACEHOLDER,
            tip: Entity::PLACEHOLDER,
        }
    }
}

pub fn spawn_hand_entities(mut commands: Commands) {
    let hands = [Hand::Left, Hand::Right];
    let bones = HandBone::get_all_bones();
    //hand resource
    let mut hand_resource = HandsResource::default();
    for hand in hands.iter() {
        for bone in bones.iter() {
            let boneid = commands
                .spawn((
                    SpatialBundle::default(),
                    *bone,
                    OpenXRTracker,
                    *hand,
                    BoneTrackingStatus::Emulated,
                    HandBoneRadius(0.1),
                ))
                .id();
            let hand = match hand {
                Hand::Left => &mut hand_resource.left,
                Hand::Right => &mut hand_resource.right,
            };
            let bone = match bone {
                HandBone::Palm => &mut hand.palm,
                HandBone::Wrist => &mut hand.wrist,
                HandBone::ThumbMetacarpal => &mut hand.thumb.metacarpal,
                HandBone::ThumbProximal => &mut hand.thumb.proximal,
                HandBone::ThumbDistal => &mut hand.thumb.distal,
                HandBone::ThumbTip => &mut hand.thumb.tip,
                HandBone::IndexMetacarpal => &mut hand.index.metacarpal,
                HandBone::IndexProximal => &mut hand.index.proximal,
                HandBone::IndexIntermediate => &mut hand.index.intermediate,
                HandBone::IndexDistal => &mut hand.index.distal,
                HandBone::IndexTip => &mut hand.index.tip,
                HandBone::MiddleMetacarpal => &mut hand.middle.metacarpal,
                HandBone::MiddleProximal => &mut hand.middle.proximal,
                HandBone::MiddleIntermediate => &mut hand.middle.intermediate,
                HandBone::MiddleDistal => &mut hand.middle.distal,
                HandBone::MiddleTip => &mut hand.middle.tip,
                HandBone::RingMetacarpal => &mut hand.ring.metacarpal,
                HandBone::RingProximal => &mut hand.ring.proximal,
                HandBone::RingIntermediate => &mut hand.ring.intermediate,
                HandBone::RingDistal => &mut hand.ring.distal,
                HandBone::RingTip => &mut hand.ring.tip,
                HandBone::LittleMetacarpal => &mut hand.little.metacarpal,
                HandBone::LittleProximal => &mut hand.little.proximal,
                HandBone::LittleIntermediate => &mut hand.little.intermediate,
                HandBone::LittleDistal => &mut hand.little.distal,
                HandBone::LittleTip => &mut hand.little.tip,
            };
            *bone = boneid;
        }
    }
    commands.insert_resource(hand_resource);
}

#[derive(Debug, Component, DerefMut, Deref)]
pub struct HandBoneRadius(pub f32);

pub fn draw_hand_entities(
    mut gizmos: Gizmos,
    query: Query<(&Transform, &HandBone, &HandBoneRadius)>,
) {
    for (transform, hand_bone, hand_bone_radius) in query.iter() {
        let (_, color) = get_bone_gizmo_style(hand_bone);
        gizmos.sphere(
            transform.translation,
            transform.rotation,
            hand_bone_radius.0,
            color,
        );
    }
}

pub(crate) fn get_bone_gizmo_style(hand_bone: &HandBone) -> (f32, Color) {
    match hand_bone {
        HandBone::Palm => (0.01, Color::WHITE),
        HandBone::Wrist => (0.01, Color::GRAY),
        HandBone::ThumbMetacarpal => (0.01, Color::RED),
        HandBone::ThumbProximal => (0.008, Color::RED),
        HandBone::ThumbDistal => (0.006, Color::RED),
        HandBone::ThumbTip => (0.004, Color::RED),
        HandBone::IndexMetacarpal => (0.01, Color::ORANGE),
        HandBone::IndexProximal => (0.008, Color::ORANGE),
        HandBone::IndexIntermediate => (0.006, Color::ORANGE),
        HandBone::IndexDistal => (0.004, Color::ORANGE),
        HandBone::IndexTip => (0.002, Color::ORANGE),
        HandBone::MiddleMetacarpal => (0.01, Color::YELLOW),
        HandBone::MiddleProximal => (0.008, Color::YELLOW),
        HandBone::MiddleIntermediate => (0.006, Color::YELLOW),
        HandBone::MiddleDistal => (0.004, Color::YELLOW),
        HandBone::MiddleTip => (0.002, Color::YELLOW),
        HandBone::RingMetacarpal => (0.01, Color::GREEN),
        HandBone::RingProximal => (0.008, Color::GREEN),
        HandBone::RingIntermediate => (0.006, Color::GREEN),
        HandBone::RingDistal => (0.004, Color::GREEN),
        HandBone::RingTip => (0.002, Color::GREEN),
        HandBone::LittleMetacarpal => (0.01, Color::BLUE),
        HandBone::LittleProximal => (0.008, Color::BLUE),
        HandBone::LittleIntermediate => (0.006, Color::BLUE),
        HandBone::LittleDistal => (0.004, Color::BLUE),
        HandBone::LittleTip => (0.002, Color::BLUE),
    }
}
