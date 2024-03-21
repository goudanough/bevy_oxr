use crate::input::XrInput;
use crate::resources::{XrInstance, XrSession};
use crate::xr_input::controllers::Handed;
use crate::xr_input::Hand;
use bevy::prelude::{Commands, Res, ResMut, Resource};
use openxr::{
    ActionSet, AnyGraphics, FrameState, Path, Posef, Session, Space, SpaceLocation, SpaceVelocity,
};

use std::sync::OnceLock;

use super::actions::{ActionHandednes, ActionType, SetupActionSets, XrActionSets, XrBinding};

pub fn post_action_setup_oculus_controller(
    action_sets: Res<XrActionSets>,
    mut controller: ResMut<OculusController>,
    instance: Res<XrInstance>,
    session: Res<XrSession>,
) {
    let s = Session::<AnyGraphics>::clone(&session);
    let left_path = instance.string_to_path("/user/hand/left").unwrap();
    let right_path = instance.string_to_path("/user/hand/right").unwrap();
    let grip_action = action_sets
        .get_action_posef("oculus_input", "hand_pose")
        .unwrap();
    let aim_action = action_sets
        .get_action_posef("oculus_input", "pointer_pose")
        .unwrap();
    controller.grip_space = Some(Handed {
        left: grip_action
            .create_space(s.clone(), left_path, Posef::IDENTITY)
            .unwrap(),
        right: grip_action
            .create_space(s.clone(), right_path, Posef::IDENTITY)
            .unwrap(),
    });
    controller.aim_space = Some(Handed {
        left: aim_action
            .create_space(s.clone(), left_path, Posef::IDENTITY)
            .unwrap(),
        right: aim_action
            .create_space(s.clone(), right_path, Posef::IDENTITY)
            .unwrap(),
    })
}

pub fn setup_oculus_controller(mut commands: Commands, action_sets: ResMut<SetupActionSets>) {
    let oculus_controller = OculusController::new(action_sets).unwrap();
    commands.insert_resource(oculus_controller);
}

#[derive(Resource, Clone)]
pub struct ActionSets(pub Vec<ActionSet>);

pub struct OculusControllerRef<'a> {
    oculus_controller: &'a OculusController,
    action_sets: &'a XrActionSets,
    session: &'a Session<AnyGraphics>,
    frame_state: &'a FrameState,
    xr_input: &'a XrInput,
}

pub static RIGHT_SUBACTION_PATH: OnceLock<Path> = OnceLock::new();
pub static LEFT_SUBACTION_PATH: OnceLock<Path> = OnceLock::new();

pub fn init_subaction_path(instance: Res<XrInstance>) {
    let _ = LEFT_SUBACTION_PATH.set(instance.string_to_path("/user/hand/left").unwrap());
    let _ = RIGHT_SUBACTION_PATH.set(instance.string_to_path("/user/hand/right").unwrap());
}

pub fn subaction_path(hand: Hand) -> Path {
    *match hand {
        Hand::Left => LEFT_SUBACTION_PATH.get().unwrap(),
        Hand::Right => RIGHT_SUBACTION_PATH.get().unwrap(),
    }
}

impl OculusControllerRef<'_> {
    fn get_button_state(&self, action_name: &'static str, path: Path) -> bool {
        self.action_sets
            .get_action_bool("oculus_input", action_name)
            .unwrap()
            .state(self.session, path)
            .map(|v| v.current_state)
            .unwrap_or(false)
    }
    fn get_axis_state(&self, action_name: &'static str, path: Path) -> f32 {
        self.action_sets
            .get_action_f32("oculus_input", action_name)
            .unwrap()
            .state(self.session, path)
            .map(|v| v.current_state)
            .unwrap_or_default()
    }
    fn get_space(
        &self,
        hand: Hand,
        f: impl Fn(&OculusController) -> Option<&Handed<Space>>,
    ) -> (SpaceLocation, SpaceVelocity) {
        let space = f(self.oculus_controller).unwrap();
        match hand {
            Hand::Left => &space.left,
            Hand::Right => &space.right,
        }
        .relate(
            &self.xr_input.stage,
            self.frame_state.predicted_display_time,
        )
        .unwrap_or_default()
    }

    pub fn grip_space(&self, hand: Hand) -> (SpaceLocation, SpaceVelocity) {
        self.get_space(hand, |controller| controller.grip_space.as_ref())
    }
    pub fn aim_space(&self, hand: Hand) -> (SpaceLocation, SpaceVelocity) {
        self.get_space(hand, |controller| controller.aim_space.as_ref())
    }
    pub fn squeeze(&self, hand: Hand) -> f32 {
        self.get_axis_state("squeeze", subaction_path(hand))
    }
    pub fn trigger(&self, hand: Hand) -> f32 {
        self.get_axis_state("trigger", subaction_path(hand))
    }
    pub fn trigger_touched(&self, hand: Hand) -> bool {
        self.get_button_state("trigger_touched", subaction_path(hand))
    }
    pub fn x_button(&self) -> bool {
        self.get_button_state("x_button", Path::NULL)
    }
    pub fn x_button_touched(&self) -> bool {
        self.get_button_state("x_button_touch", Path::NULL)
    }
    pub fn y_button(&self) -> bool {
        self.get_button_state("y_button", Path::NULL)
    }
    pub fn y_button_touched(&self) -> bool {
        self.get_button_state("y_button_touch", Path::NULL)
    }
    pub fn menu_button(&self) -> bool {
        self.get_button_state("menu_button", Path::NULL)
    }
    pub fn a_button(&self) -> bool {
        self.get_button_state("a_button", Path::NULL)
    }
    pub fn a_button_touched(&self) -> bool {
        self.get_button_state("a_button_touch", Path::NULL)
    }
    pub fn b_button(&self) -> bool {
        self.get_button_state("b_button", Path::NULL)
    }
    pub fn b_button_touched(&self) -> bool {
        self.get_button_state("b_button_touch", Path::NULL)
    }
    pub fn thumbstick_touch(&self, hand: Hand) -> bool {
        self.get_button_state("thumbstick_touch", subaction_path(hand))
    }
    pub fn thumbstick(&self, hand: Hand) -> Thumbstick {
        Thumbstick {
            x: self.get_axis_state("thumbstick_x", subaction_path(hand)),
            y: self.get_axis_state("thumbstick_y", subaction_path(hand)),
            click: self.get_button_state("thumbstick_click", subaction_path(hand)),
        }
    }
    pub fn thumbrest_touch(&self, hand: Hand) -> bool {
        self.get_button_state("thumbrest_touch", subaction_path(hand))
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Thumbstick {
    pub x: f32,
    pub y: f32,
    pub click: bool,
}

impl OculusController {
    pub fn get_ref<'a>(
        &'a self,
        session: &'a Session<AnyGraphics>,
        frame_state: &'a FrameState,
        xr_input: &'a XrInput,
        action_sets: &'a XrActionSets,
    ) -> OculusControllerRef {
        OculusControllerRef {
            oculus_controller: self,
            session,
            frame_state,
            xr_input,
            action_sets,
        }
    }
}

#[derive(Resource)]
pub struct OculusController {
    pub grip_space: Option<Handed<Space>>,
    pub aim_space: Option<Handed<Space>>,
}
impl OculusController {
    pub fn new(mut action_sets: ResMut<SetupActionSets>) -> anyhow::Result<Self> {
        let action_set =
            action_sets.add_action_set("oculus_input", "Oculus Touch Controller Input".into(), 0);
        action_set.new_action(
            "hand_pose",
            "Hand Pose".into(),
            ActionType::PoseF,
            ActionHandednes::Double,
        );
        action_set.new_action(
            "pointer_pose",
            "Pointer Pose".into(),
            ActionType::PoseF,
            ActionHandednes::Double,
        );
        action_set.new_action(
            "squeeze",
            "Grip Pull".into(),
            ActionType::F32,
            ActionHandednes::Double,
        );
        action_set.new_action(
            "trigger",
            "Trigger Pull".into(),
            ActionType::F32,
            ActionHandednes::Double,
        );
        action_set.new_action(
            "trigger_touched",
            "Trigger Touch".into(),
            ActionType::Bool,
            ActionHandednes::Double,
        );
        action_set.new_action(
            "haptic_feedback",
            "Haptic Feedback".into(),
            ActionType::Haptic,
            ActionHandednes::Double,
        );
        action_set.new_action(
            "x_button",
            "X Button".into(),
            ActionType::Bool,
            ActionHandednes::Single,
        );
        action_set.new_action(
            "x_button_touch",
            "X Button Touch".into(),
            ActionType::Bool,
            ActionHandednes::Single,
        );
        action_set.new_action(
            "y_button",
            "Y Button".into(),
            ActionType::Bool,
            ActionHandednes::Single,
        );
        action_set.new_action(
            "y_button_touch",
            "Y Button Touch".into(),
            ActionType::Bool,
            ActionHandednes::Single,
        );
        action_set.new_action(
            "a_button",
            "A Button".into(),
            ActionType::Bool,
            ActionHandednes::Single,
        );
        action_set.new_action(
            "a_button_touch",
            "A Button Touch".into(),
            ActionType::Bool,
            ActionHandednes::Single,
        );
        action_set.new_action(
            "b_button",
            "B Button".into(),
            ActionType::Bool,
            ActionHandednes::Single,
        );
        action_set.new_action(
            "b_button_touch",
            "B Button Touch".into(),
            ActionType::Bool,
            ActionHandednes::Single,
        );
        action_set.new_action(
            "menu_button",
            "Menu Button".into(),
            ActionType::Bool,
            ActionHandednes::Single,
        );
        action_set.new_action(
            "thumbstick_x",
            "Thumbstick X".into(),
            ActionType::F32,
            ActionHandednes::Double,
        );
        action_set.new_action(
            "thumbstick_y",
            "Thumbstick y".into(),
            ActionType::F32,
            ActionHandednes::Double,
        );
        action_set.new_action(
            "thumbstick_touch",
            "Thumbstick Touch".into(),
            ActionType::Bool,
            ActionHandednes::Double,
        );
        action_set.new_action(
            "thumbstick_click",
            "Thumbstick Click".into(),
            ActionType::Bool,
            ActionHandednes::Double,
        );
        action_set.new_action(
            "thumbrest_touch",
            "Thumbrest Touch".into(),
            ActionType::Bool,
            ActionHandednes::Double,
        );

        let this = OculusController {
            grip_space: None,
            aim_space: None,
        };
        action_set.suggest_binding(
            "/interaction_profiles/oculus/touch_controller",
            &[
                XrBinding::new("hand_pose", "/user/hand/left/input/grip/pose"),
                XrBinding::new("hand_pose", "/user/hand/right/input/grip/pose"),
                XrBinding::new("pointer_pose", "/user/hand/left/input/aim/pose"),
                XrBinding::new("pointer_pose", "/user/hand/right/input/aim/pose"),
                XrBinding::new("squeeze", "/user/hand/left/input/squeeze/value"),
                XrBinding::new("squeeze", "/user/hand/right/input/squeeze/value"),
                XrBinding::new("trigger", "/user/hand/left/input/trigger/value"),
                XrBinding::new("trigger", "/user/hand/right/input/trigger/value"),
                XrBinding::new("trigger_touched", "/user/hand/left/input/trigger/touch"),
                XrBinding::new("trigger_touched", "/user/hand/right/input/trigger/touch"),
                XrBinding::new("haptic_feedback", "/user/hand/left/output/haptic"),
                XrBinding::new("haptic_feedback", "/user/hand/right/output/haptic"),
                XrBinding::new("x_button", "/user/hand/left/input/x/click"),
                XrBinding::new("x_button_touch", "/user/hand/left/input/x/touch"),
                XrBinding::new("y_button", "/user/hand/left/input/y/click"),
                XrBinding::new("y_button_touch", "/user/hand/left/input/y/touch"),
                XrBinding::new("a_button", "/user/hand/right/input/a/click"),
                XrBinding::new("a_button_touch", "/user/hand/right/input/a/touch"),
                XrBinding::new("b_button", "/user/hand/right/input/b/click"),
                XrBinding::new("b_button_touch", "/user/hand/right/input/b/touch"),
                XrBinding::new("menu_button", "/user/hand/left/input/menu/click"),
                XrBinding::new("thumbstick_x", "/user/hand/left/input/thumbstick/x"),
                XrBinding::new("thumbstick_y", "/user/hand/left/input/thumbstick/y"),
                XrBinding::new("thumbstick_x", "/user/hand/right/input/thumbstick/x"),
                XrBinding::new("thumbstick_y", "/user/hand/right/input/thumbstick/y"),
                XrBinding::new("thumbstick_click", "/user/hand/left/input/thumbstick/click"),
                XrBinding::new(
                    "thumbstick_click",
                    "/user/hand/right/input/thumbstick/click",
                ),
                XrBinding::new("thumbstick_touch", "/user/hand/left/input/thumbstick/touch"),
                XrBinding::new(
                    "thumbstick_touch",
                    "/user/hand/right/input/thumbstick/touch",
                ),
                XrBinding::new("thumbrest_touch", "/user/hand/left/input/thumbrest/touch"),
                XrBinding::new("thumbrest_touch", "/user/hand/right/input/thumbrest/touch"),
            ],
        );
        Ok(this)
    }
}
