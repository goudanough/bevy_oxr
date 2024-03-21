use bevy::log::info;
use bevy::prelude::{
    Color, Component, Entity, Event, EventReader, EventWriter, Gizmos, GlobalTransform, Quat,
    Query, Transform, Vec3, With, Without,
};

use super::trackers::{AimPose, OpenXRTrackingRoot};

#[derive(Component)]
pub struct XRDirectInteractor;

#[derive(Component)]
pub struct XRRayInteractor;

#[derive(Component)]
pub struct XRSocketInteractor;

#[derive(Component)]
pub struct Touched(pub bool);

#[derive(Component, Clone, Copy, PartialEq, PartialOrd, Debug, Default)]
pub enum XRInteractableState {
    #[default]
    Idle,
    Hover,
    Select,
}

#[derive(Component, Default)]
pub enum XRInteractorState {
    #[default]
    Idle,
    Selecting,
}

#[derive(Component, Default)]
pub enum XRSelection {
    #[default]
    Empty,
    Full(Entity),
}

#[derive(Component)]
pub struct XRInteractable;

pub fn draw_socket_gizmos(
    mut gizmos: Gizmos,
    interactor_query: Query<(
        &GlobalTransform,
        &XRInteractorState,
        Entity,
        &XRSocketInteractor,
    )>,
) {
    for (global, state, _entity, _socket) in interactor_query.iter() {
        let mut transform = global.compute_transform();
        transform.scale = Vec3::splat(0.1);
        let color = match state {
            XRInteractorState::Idle => Color::BLUE,
            XRInteractorState::Selecting => Color::PURPLE,
        };
        gizmos.cuboid(transform, color)
    }
}

pub fn draw_interaction_gizmos(
    mut gizmos: Gizmos,
    interactable_query: Query<
        (&GlobalTransform, &XRInteractableState),
        (With<XRInteractable>, Without<XRDirectInteractor>),
    >,
    interactor_query: Query<
        (
            &GlobalTransform,
            &XRInteractorState,
            Option<&XRDirectInteractor>,
            Option<&XRRayInteractor>,
            Option<&AimPose>,
        ),
        Without<XRInteractable>,
    >,
    tracking_root_query: Query<&mut Transform, With<OpenXRTrackingRoot>>,
) {
    let root = tracking_root_query.get_single().unwrap();
    for (global_transform, interactable_state) in interactable_query.iter() {
        let transform = global_transform.compute_transform();
        let color = match interactable_state {
            XRInteractableState::Idle => Color::RED,
            XRInteractableState::Hover => Color::YELLOW,
            XRInteractableState::Select => Color::GREEN,
        };
        gizmos.sphere(transform.translation, transform.rotation, 0.1, color);
    }

    for (interactor_global_transform, interactor_state, direct, ray, aim) in interactor_query.iter()
    {
        let transform = interactor_global_transform.compute_transform();
        if direct.is_some() {
            let mut local = transform;
            local.scale = Vec3::splat(0.1);
            let quat = Quat::from_euler(
                bevy::prelude::EulerRot::XYZ,
                45_f32.to_radians(),
                0.0,
                45_f32.to_radians(),
            );
            local.rotation = quat;
            let color = match interactor_state {
                XRInteractorState::Idle => Color::BLUE,
                XRInteractorState::Selecting => Color::PURPLE,
            };
            gizmos.cuboid(local, color);
        }
        if ray.is_some() {
            if let Some(aim) = aim {
                let color = match interactor_state {
                    XRInteractorState::Idle => Color::BLUE,
                    XRInteractorState::Selecting => Color::PURPLE,
                };
                gizmos.ray(
                    root.translation + root.rotation.mul_vec3(aim.0.translation),
                    root.rotation.mul_vec3(*aim.0.forward()),
                    color,
                );
            } else {
                todo!()
            }
        }
    }
}

#[derive(Event)]
pub struct InteractionEvent {
    pub interactor: Entity,
    pub interactable: Entity,
    pub interactable_state: XRInteractableState,
}

pub fn socket_interactions(
    interactable_query: Query<
        (&GlobalTransform, &mut XRInteractableState, Entity),
        (With<XRInteractable>, Without<XRSocketInteractor>),
    >,
    interactor_query: Query<
        (
            &GlobalTransform,
            &XRInteractorState,
            Entity,
            &XRSocketInteractor,
        ),
        Without<XRInteractable>,
    >,
    mut writer: EventWriter<InteractionEvent>,
) {
    for interactable in interactable_query.iter() {
        //for the interactables
        for socket in interactor_query.iter() {
            let interactor_global_transform = socket.0;
            let xr_interactable_global_transform = interactable.0;
            let interactor_state = socket.1;
            //check for sphere overlaps
            let size = 0.1;
            if interactor_global_transform
                .compute_transform()
                .translation
                .distance_squared(
                    xr_interactable_global_transform
                        .compute_transform()
                        .translation,
                )
                < (size * size) * 2.0
            {
                //check for selections first
                match interactor_state {
                    XRInteractorState::Idle => {
                        let event = InteractionEvent {
                            interactor: socket.2,
                            interactable: interactable.2,
                            interactable_state: XRInteractableState::Hover,
                        };
                        writer.send(event);
                    }
                    XRInteractorState::Selecting => {
                        let event = InteractionEvent {
                            interactor: socket.2,
                            interactable: interactable.2,
                            interactable_state: XRInteractableState::Select,
                        };
                        writer.send(event);
                    }
                }
            }
        }
    }
}

pub fn interactions(
    interactable_query: Query<
        (&GlobalTransform, Entity),
        (With<XRInteractable>, Without<XRDirectInteractor>),
    >,
    interactor_query: Query<
        (
            &GlobalTransform,
            &XRInteractorState,
            Entity,
            Option<&XRDirectInteractor>,
            Option<&XRRayInteractor>,
            Option<&AimPose>,
        ),
        Without<XRInteractable>,
    >,
    tracking_root_query: Query<&mut Transform, With<OpenXRTrackingRoot>>,
    mut writer: EventWriter<InteractionEvent>,
) {
    for (xr_interactable_global_transform, interactable_entity) in interactable_query.iter() {
        for (interactor_global_transform, interactor_state, interactor_entity, direct, ray, aim) in
            interactor_query.iter()
        {
            if direct.is_some() {
                //check for sphere overlaps
                let size = 0.1;
                if interactor_global_transform
                    .compute_transform()
                    .translation
                    .distance_squared(
                        xr_interactable_global_transform
                            .compute_transform()
                            .translation,
                    )
                    < (size * size) * 2.0
                {
                    //check for selections first
                    let event = InteractionEvent {
                        interactor: interactor_entity,
                        interactable: interactable_entity,
                        interactable_state: match XRInteractorState::Idle {
                            XRInteractorState::Idle => XRInteractableState::Hover,
                            XRInteractorState::Selecting => XRInteractableState::Select,
                        },
                    };
                    writer.send(event);
                }
            }

            if ray.is_some() {
                //check for ray-sphere intersection
                let sphere_transform = xr_interactable_global_transform.compute_transform();
                let center = sphere_transform.translation;
                let radius: f32 = 0.1;
                //I hate this but the aim pose needs the root for now
                let root = tracking_root_query.get_single().unwrap();
                if let Some(aim) = aim {
                    let ray_origin = root.translation + root.rotation.mul_vec3(aim.0.translation);
                    let ray_dir = root.rotation.mul_vec3(*aim.0.forward()).normalize_or_zero();

                    if ray_sphere_intersection(center, radius, ray_origin, ray_dir) {
                        let event = InteractionEvent {
                            interactor: interactor_entity,
                            interactable: interactable_entity,
                            interactable_state: match interactor_state {
                                XRInteractorState::Idle => XRInteractableState::Hover,
                                XRInteractorState::Selecting => XRInteractableState::Select,
                            },
                        };
                        writer.send(event);
                    }
                } else {
                    info!("no aim pose")
                }
            }
        }
    }
}

pub fn update_interactable_states(
    mut events: EventReader<InteractionEvent>,
    mut interactable_query: Query<
        (Entity, &mut XRInteractableState, &mut Touched),
        With<XRInteractable>,
    >,
) {
    //i very much dislike this
    for (_entity, _state, mut touched) in interactable_query.iter_mut() {
        *touched = Touched(false);
    }
    for event in events.read() {
        //lets change the state
        if let Ok((_entity, mut entity_state, mut touched)) =
            interactable_query.get_mut(event.interactable)
        {
            //since we have an event we were touched this frame, i hate this name
            *touched = Touched(true);
            if event.interactable_state > *entity_state {
                // info!(
                //     "event.state: {:?}, interactable.state: {:?}",
                //     event.interactable_state, entity_state
                // );
                // info!("event has a higher state");
            }
            *entity_state = event.interactable_state;
        }
    }
    //lets go through all the untouched interactables and set them to idle
    for (_entity, mut state, touched) in interactable_query.iter_mut() {
        if !touched.0 {
            *state = XRInteractableState::Idle;
        }
    }
}

fn ray_sphere_intersection(center: Vec3, radius: f32, ray_origin: Vec3, ray_dir: Vec3) -> bool {
    let l = center - ray_origin;
    let adj = l.dot(ray_dir);
    let d2 = l.dot(l) - (adj * adj);
    let radius2 = radius * radius;
    if d2 > radius2 {
        return false;
    }
    let thc = (radius2 - d2).sqrt();
    let t0 = adj - thc;
    let t1 = adj + thc;

    if t0 < 0.0 && t1 < 0.0 {
        return false;
    }

    // let distance = if t0 < t1 { t0 } else { t1 };
    true
}
