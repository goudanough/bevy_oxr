use std::sync::Arc;

use bevy::{prelude::*, render::extract_resource::ExtractResource};
use openxr as xr;

#[derive(Clone, Resource, ExtractResource)]
pub struct XrInput {
    //pub action_set: xr::ActionSet,
    //pub hand_pose: xr::Action<xr::Posef>,
    //pub right_space: Arc<xr::Space>,
    //pub left_space: Arc<xr::Space>,
    pub stage: Arc<xr::Space>,
    pub head: Arc<xr::Space>,
}

impl XrInput {
    pub fn new(
        instance: xr::Instance,
        session: xr::Session<xr::AnyGraphics>,
        // frame_state: &FrameState,
    ) -> xr::Result<Self> {
        // let right_hand_subaction_path = instance.string_to_path("/user/hand/right").unwrap();
        // let right_hand_grip_pose_path = instance
        //     .string_to_path("/user/hand/right/input/grip/pose")
        //     .unwrap();
        // let hand_pose = action_set.create_action::<xr::Posef>(
        //     "hand_pose",
        //     "Hand Pose",
        //     &[left_hand_subaction_path, right_hand_subaction_path],
        // )?;
        // /* let left_action =
        // action_set.create_action::<xr::Posef>("left_hand", "Left Hand Controller", &[])?;*/
        // instance.suggest_interaction_profile_bindings(
        //     instance.string_to_path("/interaction_profiles/khr/simple_controller")?,
        //     &[
        //         xr::Binding::new(&hand_pose, right_hand_grip_pose_path),
        //         xr::Binding::new(&hand_pose, left_hand_grip_pose_path),
        //     ],
        // )?;
        //
        // let right_space = hand_pose.create_space(
        //     session.clone(),
        //     right_hand_subaction_path,
        //     xr::Posef::IDENTITY,
        // )?;
        // let left_space = hand_pose.create_space(
        //     session.clone(),
        //     left_hand_subaction_path,
        //     xr::Posef::IDENTITY,
        // )?;

        let stage = match instance.exts().ext_local_floor {
            None => session
                .create_reference_space(xr::ReferenceSpaceType::STAGE, xr::Posef::IDENTITY)?,
            Some(_) => session.create_reference_space(
                xr::ReferenceSpaceType::LOCAL_FLOOR_EXT,
                xr::Posef::IDENTITY,
            )?,
        };
        let head =
            session.create_reference_space(xr::ReferenceSpaceType::VIEW, xr::Posef::IDENTITY)?;
        // let y = stage
        //     .locate(&head, frame_state.predicted_display_time).unwrap()
        //     .pose
        //     .position
        //     .y;
        // let local = session.create_reference_space(
        //     xr::ReferenceSpaceType::LOCAL,
        //     xr::Posef {
        //         position: xr::Vector3f { x: 0.0, y, z: 0.0 },
        //         orientation: xr::Quaternionf::IDENTITY,
        //     },
        // ).unwrap();
        //session.attach_action_sets(&[&action_set])?;
        //session.attach_action_sets(&[])?;
        Ok(Self {
            //action_set,
            //hand_pose,
            // right_space: Arc::new(right_space),
            // left_space: Arc::new(left_space),
            stage: Arc::new(stage),
            head: Arc::new(head),
        })
    }
}
