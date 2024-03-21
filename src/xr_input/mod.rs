pub mod actions;
pub mod controllers;
pub mod debug_gizmos;
pub mod hand_poses;
pub mod hands;
pub mod interactions;
pub mod oculus_touch;
pub mod prototype_locomotion;
pub mod trackers;
pub mod xr_camera;

use crate::resources::XrSession;
use crate::xr_begin_frame;
use crate::xr_init::{xr_only, XrPostSetup, XrPreSetup, XrSetup};
use crate::xr_input::controllers::XrControllerType;
use crate::xr_input::oculus_touch::setup_oculus_controller;
use crate::xr_input::xr_camera::{xr_camera_head_sync, Eye, XRProjection, XrCameraBundle};
use bevy::app::{App, PostUpdate};
use bevy::ecs::entity::Entity;
use bevy::ecs::query::With;
use bevy::ecs::system::Query;
use bevy::log::warn;
use bevy::math::Vec2;
use bevy::prelude::{BuildChildren, Component, Deref, DerefMut, IntoSystemConfigs, Resource};
use bevy::prelude::{Commands, Plugin, PreUpdate, Quat, Res, SpatialBundle, Update, Vec3};
use bevy::render::camera::CameraProjectionPlugin;
use bevy::render::view::{update_frusta, VisibilitySystems};
use bevy::transform::TransformSystem;
use bevy::utils::HashMap;
use openxr::Binding;

use self::actions::OpenXrActionsPlugin;
use self::oculus_touch::{init_subaction_path, post_action_setup_oculus_controller, ActionSets};
use self::trackers::{
    adopt_open_xr_trackers, update_open_xr_controllers, OpenXRLeftEye, OpenXRRightEye,
    OpenXRTrackingRoot,
};

#[derive(Copy, Clone)]
pub struct OpenXrInput {
    pub controller_type: XrControllerType,
}
#[derive(Clone, Copy, Debug, Ord, PartialOrd, Eq, PartialEq, Component)]
pub enum Hand {
    Left,
    Right,
}

impl OpenXrInput {
    pub fn new(controller_type: XrControllerType) -> Self {
        Self { controller_type }
    }
}

impl Plugin for OpenXrInput {
    fn build(&self, app: &mut App) {
        app.add_plugins(CameraProjectionPlugin::<XRProjection>::default());
        app.add_plugins(OpenXrActionsPlugin);
        app.add_systems(XrPostSetup, post_action_setup_oculus_controller);
        match self.controller_type {
            XrControllerType::OculusTouch => {
                app.add_systems(XrSetup, setup_oculus_controller);
            }
        }
        //adopt any new trackers
        app.add_systems(PreUpdate, adopt_open_xr_trackers.run_if(xr_only()));
        app.add_systems(PreUpdate, action_set_system.run_if(xr_only()));
        app.add_systems(
            PreUpdate,
            xr_camera_head_sync.run_if(xr_only()).after(xr_begin_frame),
        );
        //update controller trackers
        app.add_systems(Update, update_open_xr_controllers.run_if(xr_only()));
        app.add_systems(
            PostUpdate,
            update_frusta::<XRProjection>
                .after(TransformSystem::TransformPropagate)
                .before(VisibilitySystems::UpdatePerspectiveFrusta),
        );
        app.add_systems(XrPreSetup, init_subaction_path);
        app.add_systems(XrSetup, setup_xr_cameras);
    }
}

#[derive(Deref, DerefMut, Resource)]
pub struct InteractionProfileBindings(pub HashMap<&'static str, Vec<Binding<'static>>>);

// fn setup_binding_recommendations(
//     mut commands: Commands,
//     instance: Res<XrInstance>,
//     bindings: Res<InteractionProfileBindings>,
// ) {
//     commands.remove_resource::<InteractionProfileBindings>();
// }

fn setup_xr_cameras(
    mut commands: Commands,
    tracking_root_query: Query<Entity, With<OpenXRTrackingRoot>>,
) {
    //this needs to do the whole xr tracking volume not just cameras
    //get the root?

    let tracking_root = match tracking_root_query.get_single() {
        Ok(e) => e,
        Err(_) => commands
            .spawn((SpatialBundle::default(), OpenXRTrackingRoot))
            .id(),
    };
    let right = commands
        .spawn((XrCameraBundle::new(Eye::Right), OpenXRRightEye))
        .id();
    let left = commands
        .spawn((XrCameraBundle::new(Eye::Left), OpenXRLeftEye))
        .id();
    commands.entity(tracking_root).push_children(&[right, left]);
}

pub fn action_set_system(action_sets: Res<ActionSets>, session: Res<XrSession>) {
    let mut active_action_sets = vec![];
    for i in &action_sets.0 {
        active_action_sets.push(openxr::ActiveActionSet::new(i));
    }
    //info!("action sets: {:#?}", action_sets.0.len());
    if let Err(err) = session.sync_actions(&active_action_sets) {
        warn!("{}", err);
    }
}

pub trait Vec2Conv {
    fn to_vec2(&self) -> Vec2;
}

impl Vec2Conv for openxr::Vector2f {
    fn to_vec2(&self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }
}
pub trait Vec3Conv {
    fn to_vec3(&self) -> Vec3;
}

impl Vec3Conv for openxr::Vector3f {
    fn to_vec3(&self) -> Vec3 {
        Vec3::new(self.x, self.y, self.z)
    }
}
pub trait QuatConv {
    fn to_quat(&self) -> Quat;
}

impl QuatConv for openxr::Quaternionf {
    fn to_quat(&self) -> Quat {
        Quat::from_xyzw(self.x, self.y, self.z, self.w)
    }
}
