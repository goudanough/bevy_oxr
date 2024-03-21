use bevy::{app::PluginGroupBuilder, prelude::*};

use self::{emulated::HandEmulationPlugin, hand_tracking::HandTrackingPlugin};

pub mod common;
pub mod emulated;
pub mod hand_tracking;

pub struct XrHandPlugins;

impl PluginGroup for XrHandPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(HandTrackingPlugin)
            .add(HandEmulationPlugin)
            .build()
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub enum BoneTrackingStatus {
    Emulated,
    Tracked,
}

#[derive(Component, Debug, Clone, Copy)]
pub enum HandBone {
    Palm,
    Wrist,
    ThumbMetacarpal,
    ThumbProximal,
    ThumbDistal,
    ThumbTip,
    IndexMetacarpal,
    IndexProximal,
    IndexIntermediate,
    IndexDistal,
    IndexTip,
    MiddleMetacarpal,
    MiddleProximal,
    MiddleIntermediate,
    MiddleDistal,
    MiddleTip,
    RingMetacarpal,
    RingProximal,
    RingIntermediate,
    RingDistal,
    RingTip,
    LittleMetacarpal,
    LittleProximal,
    LittleIntermediate,
    LittleDistal,
    LittleTip,
}
impl HandBone {
    pub fn is_finger(&self) -> bool {
        !matches!(self, HandBone::Wrist | HandBone::Palm)
    }
    pub fn is_metacarpal(&self) -> bool {
        matches!(
            self,
            HandBone::ThumbMetacarpal
                | HandBone::IndexMetacarpal
                | HandBone::MiddleMetacarpal
                | HandBone::RingMetacarpal
                | HandBone::LittleMetacarpal
        )
    }
    pub const fn get_all_bones() -> [HandBone; 26] {
        [
            HandBone::Palm,
            HandBone::Wrist,
            HandBone::ThumbMetacarpal,
            HandBone::ThumbProximal,
            HandBone::ThumbDistal,
            HandBone::ThumbTip,
            HandBone::IndexMetacarpal,
            HandBone::IndexProximal,
            HandBone::IndexIntermediate,
            HandBone::IndexDistal,
            HandBone::IndexTip,
            HandBone::MiddleMetacarpal,
            HandBone::MiddleProximal,
            HandBone::MiddleIntermediate,
            HandBone::MiddleDistal,
            HandBone::MiddleTip,
            HandBone::RingMetacarpal,
            HandBone::RingProximal,
            HandBone::RingIntermediate,
            HandBone::RingDistal,
            HandBone::RingTip,
            HandBone::LittleMetacarpal,
            HandBone::LittleProximal,
            HandBone::LittleIntermediate,
            HandBone::LittleDistal,
            HandBone::LittleTip,
        ]
    }
}
