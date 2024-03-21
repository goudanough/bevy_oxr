use bevy::prelude::*;
use openxr::{HandTracker, Result, SpaceLocationFlags};

use super::common::HandBoneRadius;
use crate::{
    input::XrInput,
    resources::{XrFrameState, XrSession},
    xr_init::xr_only,
    xr_input::{hands::HandBone, trackers::OpenXRTrackingRoot, Hand, QuatConv, Vec3Conv},
};

use super::BoneTrackingStatus;

#[derive(Resource, PartialEq)]
pub enum DisableHandTracking {
    OnlyLeft,
    OnlyRight,
    Both,
}
pub struct HandTrackingPlugin;

#[derive(Resource)]
pub struct HandTrackingData {
    left_hand: HandTracker,
    right_hand: HandTracker,
}

impl HandTrackingData {
    pub fn new(session: &XrSession) -> Result<HandTrackingData> {
        let left = session.create_hand_tracker(openxr::HandEXT::LEFT)?;
        let right = session.create_hand_tracker(openxr::HandEXT::RIGHT)?;
        Ok(HandTrackingData {
            left_hand: left,
            right_hand: right,
        })
    }
    pub fn get_ref<'a>(
        &'a self,
        input: &'a XrInput,
        frame_state: &'a XrFrameState,
    ) -> HandTrackingRef<'a> {
        HandTrackingRef {
            tracking: self,
            input,
            frame_state,
        }
    }
}

pub struct HandTrackingRef<'a> {
    tracking: &'a HandTrackingData,
    input: &'a XrInput,
    frame_state: &'a XrFrameState,
}
#[derive(Debug)]
pub struct HandJoint {
    pub position: Vec3,
    pub position_valid: bool,
    pub position_tracked: bool,
    pub orientation: Quat,
    pub orientation_valid: bool,
    pub orientation_tracked: bool,
    pub radius: f32,
}

pub struct HandJoints {
    inner: [HandJoint; 26],
}
impl HandJoints {
    pub fn new(inner: [HandJoint; 26]) -> Self {
        Self { inner }
    }
    pub fn inner(&self) -> &[HandJoint; 26] {
        &self.inner
    }
    pub fn get_joint(&self, bone: HandBone) -> &HandJoint {
        &self.inner[bone as usize]
    }
}

impl<'a> HandTrackingRef<'a> {
    pub fn get_poses(&self, side: Hand) -> Option<HandJoints> {
        self.input
            .stage
            .locate_hand_joints(
                match side {
                    Hand::Left => &self.tracking.left_hand,
                    Hand::Right => &self.tracking.right_hand,
                },
                self.frame_state.predicted_display_time,
            )
            .unwrap()
            .map(|joints| {
                joints.map(|joint| HandJoint {
                    position: joint.pose.position.to_vec3(),
                    orientation: joint.pose.orientation.to_quat(),
                    position_valid: joint
                        .location_flags
                        .contains(SpaceLocationFlags::POSITION_VALID),
                    position_tracked: joint
                        .location_flags
                        .contains(SpaceLocationFlags::POSITION_TRACKED),
                    orientation_valid: joint
                        .location_flags
                        .contains(SpaceLocationFlags::ORIENTATION_VALID),
                    orientation_tracked: joint
                        .location_flags
                        .contains(SpaceLocationFlags::ORIENTATION_TRACKED),
                    radius: joint.radius,
                })
            })
            .map(HandJoints::new)
    }
}

impl Plugin for HandTrackingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            (
                update_hand_bones
                    .run_if(|dh: Option<Res<DisableHandTracking>>| {
                        !dh.is_some_and(|v| *v == DisableHandTracking::Both)
                    })
                    .run_if(xr_only()),
                update_tracking_state_on_disable,
            ),
        );
    }
}

fn update_tracking_state_on_disable(
    mut is_off: Local<bool>,
    disabled_tracking: Option<Res<DisableHandTracking>>,
    mut tracking_states: Query<&mut BoneTrackingStatus>,
) {
    if !*is_off
        && disabled_tracking
            .as_ref()
            .is_some_and(|t| **t == DisableHandTracking::Both)
    {
        tracking_states
            .par_iter_mut()
            .for_each(|mut state| *state = BoneTrackingStatus::Emulated);
    }
    *is_off = disabled_tracking
        .as_ref()
        .is_some_and(|t| **t == DisableHandTracking::Both);
}

pub fn update_hand_bones(
    disabled_tracking: Option<Res<DisableHandTracking>>,
    hand_tracking: Option<Res<HandTrackingData>>,
    xr_input: Res<XrInput>,
    xr_frame_state: Res<XrFrameState>,
    root_query: Query<&Transform, (With<OpenXRTrackingRoot>, Without<HandBone>)>,
    mut bones: Query<(
        &mut Transform,
        &Hand,
        &HandBone,
        &mut HandBoneRadius,
        &mut BoneTrackingStatus,
    )>,
) {
    let Some(hand_ref) = hand_tracking else {
        warn!("No Handtracking data!");
        return;
    };
    let hand_ref = hand_ref.get_ref(&xr_input, &xr_frame_state);

    let root_transform = root_query.get_single().unwrap();
    let left_hand_data = hand_ref.get_poses(Hand::Left);
    let right_hand_data = hand_ref.get_poses(Hand::Right);
    let disabled_tracking = disabled_tracking.as_ref().map(Res::as_ref);
    bones
        .par_iter_mut()
        .for_each(|(mut transform, hand, bone, mut radius, mut status)| {
            use DisableHandTracking::*;
            if let (Hand::Left, Some(OnlyLeft | OnlyRight)) = (&hand, disabled_tracking) {
                *status = BoneTrackingStatus::Emulated;
                return;
            }
            let bone_data = match (hand, &left_hand_data, &right_hand_data) {
                (Hand::Left, Some(data), _) | (Hand::Right, _, Some(data)) => data.get_joint(*bone),
                _ => {
                    *status = BoneTrackingStatus::Emulated;
                    return;
                }
            };
            if *status == BoneTrackingStatus::Emulated {
                *status = BoneTrackingStatus::Tracked;
            }
            radius.0 = bone_data.radius;
            *transform = transform
                .with_translation(root_transform.transform_point(bone_data.position))
                .with_rotation(root_transform.rotation * bone_data.orientation)
        });
}
