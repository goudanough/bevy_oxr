use bevy::prelude::*;
use openxr::{sys, ActionTy, HandJoint};

use super::common::{get_bone_gizmo_style, HandBoneRadius};
use crate::{
    resources::{XrInstance, XrSession},
    xr_init::{xr_only, XrSetup},
    xr_input::{
        actions::{
            ActionHandednes, ActionType, SetupActionSet, SetupActionSets, XrActionSets, XrBinding,
        },
        hand_poses::get_simulated_open_hand_transforms,
        trackers::{OpenXRLeftController, OpenXRRightController, OpenXRTrackingRoot},
        Hand,
    },
};

use super::{BoneTrackingStatus, HandBone};

pub enum TouchValue<T: ActionTy> {
    None,
    Touched(T),
}

pub struct HandEmulationPlugin;

impl Plugin for HandEmulationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_hand_skeleton_from_emulated.run_if(xr_only()));
        app.add_systems(XrSetup, setup_hand_emulation_action_set);
    }
}

const HAND_ACTION_SET: &str = "hand_pose_approx";

fn setup_hand_emulation_action_set(mut action_sets: ResMut<SetupActionSets>) {
    let action_set =
        action_sets.add_action_set(HAND_ACTION_SET, "Hand Pose Approximaiton".into(), 0);
    action_set.new_action(
        "thumb_touch",
        "Thumb Touched".into(),
        ActionType::Bool,
        ActionHandednes::Double,
    );
    action_set.new_action(
        "thumb_x",
        "Thumb X".into(),
        ActionType::F32,
        ActionHandednes::Double,
    );
    action_set.new_action(
        "thumb_y",
        "Thumb Y".into(),
        ActionType::F32,
        ActionHandednes::Double,
    );

    action_set.new_action(
        "index_touch",
        "Index Finger Touched".into(),
        ActionType::Bool,
        ActionHandednes::Double,
    );
    action_set.new_action(
        "index_value",
        "Index Finger Pull".into(),
        ActionType::F32,
        ActionHandednes::Double,
    );

    action_set.new_action(
        "middle_value",
        "Middle Finger Pull".into(),
        ActionType::F32,
        ActionHandednes::Double,
    );
    action_set.new_action(
        "ring_value",
        "Ring Finger Pull".into(),
        ActionType::F32,
        ActionHandednes::Double,
    );
    action_set.new_action(
        "little_value",
        "Little Finger Pull".into(),
        ActionType::F32,
        ActionHandednes::Double,
    );

    suggest_oculus_touch_profile(action_set);
}

pub struct EmulatedHandPoseData {}

fn suggest_oculus_touch_profile(action_set: &mut SetupActionSet) {
    action_set.suggest_binding(
        "/interaction_profiles/oculus/touch_controller",
        &[
            XrBinding::new("thumb_x", "/user/hand/left/input/thumbstick/x"),
            XrBinding::new("thumb_x", "/user/hand/right/input/thumbstick/x"),
            XrBinding::new("thumb_y", "/user/hand/left/input/thumbstick/y"),
            XrBinding::new("thumb_y", "/user/hand/right/input/thumbstick/y"),
            XrBinding::new("thumb_touch", "/user/hand/left/input/thumbstick/touch"),
            XrBinding::new("thumb_touch", "/user/hand/right/input/thumbstick/touch"),
            XrBinding::new("thumb_touch", "/user/hand/left/input/x/touch"),
            XrBinding::new("thumb_touch", "/user/hand/left/input/y/touch"),
            XrBinding::new("thumb_touch", "/user/hand/right/input/a/touch"),
            XrBinding::new("thumb_touch", "/user/hand/right/input/b/touch"),
            XrBinding::new("thumb_touch", "/user/hand/left/input/thumbrest/touch"),
            XrBinding::new("thumb_touch", "/user/hand/right/input/thumbrest/touch"),
            XrBinding::new("index_touch", "/user/hand/left/input/trigger/touch"),
            XrBinding::new("index_value", "/user/hand/left/input/trigger/value"),
            XrBinding::new("index_touch", "/user/hand/right/input/trigger/touch"),
            XrBinding::new("index_value", "/user/hand/right/input/trigger/value"),
            XrBinding::new("middle_value", "/user/hand/left/input/squeeze/value"),
            XrBinding::new("middle_value", "/user/hand/right/input/squeeze/value"),
            XrBinding::new("ring_value", "/user/hand/left/input/squeeze/value"),
            XrBinding::new("ring_value", "/user/hand/right/input/squeeze/value"),
            XrBinding::new("little_value", "/user/hand/left/input/squeeze/value"),
            XrBinding::new("little_value", "/user/hand/right/input/squeeze/value"),
        ],
    );
}

#[allow(clippy::type_complexity)]
pub(crate) fn update_hand_skeleton_from_emulated(
    session: Res<XrSession>,
    instance: Res<XrInstance>,
    action_sets: Res<XrActionSets>,
    left_controller_transform: Query<&Transform, With<OpenXRLeftController>>,
    right_controller_transform: Query<&Transform, With<OpenXRRightController>>,
    tracking_root_transform: Query<&Transform, With<OpenXRTrackingRoot>>,
    mut bones: Query<
        (
            &mut Transform,
            &HandBone,
            &Hand,
            &BoneTrackingStatus,
            &mut HandBoneRadius,
        ),
        (
            Without<OpenXRLeftController>,
            Without<OpenXRRightController>,
            Without<OpenXRTrackingRoot>,
        ),
    >,
) {
    //get the transforms outside the loop
    let left = left_controller_transform.get_single();
    let right = right_controller_transform.get_single();
    let mut data: [[Transform; 26]; 2] = [[Transform::default(); 26]; 2];
    for (subaction_path, hand) in [
        (
            instance.string_to_path("/user/hand/left").unwrap(),
            Hand::Left,
        ),
        (
            instance.string_to_path("/user/hand/right").unwrap(),
            Hand::Right,
        ),
    ] {
        let get_axis = |action_name| {
            action_sets
                .get_action_f32(HAND_ACTION_SET, action_name)
                .unwrap()
                .state(&session, subaction_path)
                .unwrap()
                .current_state
        };
        let get_state = |action_name| {
            action_sets
                .get_action_bool(HAND_ACTION_SET, action_name)
                .unwrap()
                .state(&session, subaction_path)
                .unwrap()
                .current_state
        };
        let thumb_curl = match get_state("thumb_touch") {
            true => 1.0,
            false => 0.0,
        };
        let index_curl = get_axis("index_value");
        let middle_curl = get_axis("middle_value");
        let ring_curl = get_axis("ring_value");
        let little_curl = get_axis("little_value");

        let hand_side = match hand {
            Hand::Left => &left,
            Hand::Right => &right,
        };
        if let Ok(hand_transform) = hand_side {
            data[hand as usize] = update_hand_bones_emulated(
                hand_transform,
                hand,
                thumb_curl,
                index_curl,
                middle_curl,
                ring_curl,
                little_curl,
            )
        } else {
            debug!("no {hand:?} controller transform for hand bone emulation")
        }
    }
    let trt = tracking_root_transform.single();
    for (mut t, bone, hand, status, mut radius) in bones.iter_mut() {
        if let BoneTrackingStatus::Tracked = status {
            continue;
        }
        radius.0 = get_bone_gizmo_style(bone).0;

        *t = data[*hand as usize][*bone as usize];
        *t = t.with_scale(trt.scale);
        *t = t.with_rotation(trt.rotation * t.rotation);
        *t = t.with_translation(trt.transform_point(t.translation));
    }
}

pub fn update_hand_bones_emulated(
    controller_transform: &Transform,
    hand: Hand,
    thumb_curl: f32,
    index_curl: f32,
    middle_curl: f32,
    ring_curl: f32,
    little_curl: f32,
) -> [Transform; 26] {
    let left_hand_rot = Quat::from_rotation_y(180_f32.to_radians());
    let hand_translation = controller_transform.translation;

    let controller_quat = match hand {
        Hand::Left => controller_transform.rotation.mul_quat(left_hand_rot),
        Hand::Right => controller_transform.rotation,
    };

    let splay_direction = match hand {
        Hand::Left => -1.0,
        Hand::Right => 1.0,
    };
    //lets make a structure to hold our calculated transforms for now
    let mut calc_transforms = [Transform::default(); 26];

    //get palm quat
    let y = Quat::from_rotation_y(-90_f32.to_radians());
    let x = Quat::from_rotation_x(-90_f32.to_radians());
    let palm_quat = controller_quat.mul_quat(y).mul_quat(x);
    //get simulated bones
    let hand_transform_array = get_simulated_open_hand_transforms(hand);

    //palm
    let palm = hand_transform_array[HandJoint::PALM];
    calc_transforms[HandJoint::PALM] = Transform {
        translation: hand_translation + palm.translation,
        ..default()
    };

    //wrist
    let wrist = hand_transform_array[HandJoint::WRIST];
    calc_transforms[HandJoint::WRIST] = Transform {
        translation: hand_translation + palm.translation + palm_quat.mul_vec3(wrist.translation),
        ..default()
    };

    let mut position_hand = |joints: &[sys::HandJointEXT], curl, _splay @ (x, y): (f32, f32)| {
        let splay_y = Quat::from_rotation_y((splay_direction * y).to_radians());
        let splay_x = Quat::from_rotation_x(x.to_radians());
        let splay_quat = palm_quat.mul_quat(splay_x).mul_quat(splay_y);

        let bone = joints[0];

        let meta = hand_transform_array[bone];
        let tm_start = hand_translation
            + palm_quat.mul_vec3(palm.translation)
            + palm_quat.mul_vec3(wrist.translation);
        let tm_vector = palm_quat.mul_vec3(meta.translation);
        //store it
        calc_transforms[bone] = Transform {
            translation: tm_start + tm_vector,
            ..default()
        };

        let mut prior_start = tm_start;
        let mut prior_quat = splay_quat;
        let mut prior_vector = tm_vector;

        for &bone in &joints[1..] {
            let curl_angle: f32 = get_bone_curl_angle(bone, curl);
            let tp_lrot = Quat::from_rotation_y((splay_direction * curl_angle).to_radians());
            let tp_quat = prior_quat.mul_quat(tp_lrot);
            let prox = hand_transform_array[bone];
            let tp_start = prior_start + prior_vector;
            let tp_vector = tp_quat.mul_vec3(prox.translation);
            //store it
            calc_transforms[bone] = Transform {
                translation: tp_start + tp_vector,
                ..default()
            };

            prior_start = tp_start;
            prior_quat = tp_quat;
            prior_vector = tp_vector;
        }
    };

    position_hand(
        &[
            HandJoint::THUMB_METACARPAL,
            HandJoint::THUMB_PROXIMAL,
            HandJoint::THUMB_DISTAL,
            HandJoint::THUMB_TIP,
        ],
        thumb_curl,
        (-35., 30.),
    );

    position_hand(
        &[
            HandJoint::INDEX_METACARPAL,
            HandJoint::INDEX_PROXIMAL,
            HandJoint::INDEX_INTERMEDIATE,
            HandJoint::INDEX_DISTAL,
            HandJoint::INDEX_TIP,
        ],
        index_curl,
        (0., 10.),
    );

    position_hand(
        &[
            HandJoint::MIDDLE_METACARPAL,
            HandJoint::MIDDLE_PROXIMAL,
            HandJoint::MIDDLE_INTERMEDIATE,
            HandJoint::MIDDLE_DISTAL,
            HandJoint::MIDDLE_TIP,
        ],
        middle_curl,
        (0., 0.),
    );

    position_hand(
        &[
            HandJoint::RING_METACARPAL,
            HandJoint::RING_PROXIMAL,
            HandJoint::RING_INTERMEDIATE,
            HandJoint::RING_DISTAL,
            HandJoint::RING_TIP,
        ],
        ring_curl,
        (0., -10.),
    );

    position_hand(
        &[
            HandJoint::LITTLE_METACARPAL,
            HandJoint::LITTLE_PROXIMAL,
            HandJoint::LITTLE_INTERMEDIATE,
            HandJoint::LITTLE_DISTAL,
            HandJoint::LITTLE_TIP,
        ],
        little_curl,
        (0., -20.),
    );

    calc_transforms
}

fn get_bone_curl_angle(bone: HandJoint, curl: f32) -> f32 {
    let mul: f32 = match bone {
        HandJoint::INDEX_PROXIMAL => 0.0,
        HandJoint::MIDDLE_PROXIMAL => 0.0,
        HandJoint::RING_PROXIMAL => 0.0,
        HandJoint::LITTLE_PROXIMAL => 0.0,
        HandJoint::THUMB_PROXIMAL => 0.0,
        HandJoint::THUMB_TIP => 0.1,
        HandJoint::THUMB_DISTAL => 0.1,
        HandJoint::THUMB_METACARPAL => 0.1,
        _ => 1.0,
    };
    -((mul * curl * 80.0) + 5.0)
}
