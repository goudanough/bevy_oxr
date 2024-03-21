#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]

pub mod graphics;
pub mod input;
pub mod passthrough;
pub mod resource_macros;
pub mod resources;
pub mod xr_init;
pub mod xr_input;

use std::sync::Arc;

use crate::xr_init::RenderRestartPlugin;
use crate::xr_input::hands::hand_tracking::DisableHandTracking;
use crate::xr_input::oculus_touch::ActionSets;
use bevy::app::{AppExit, PluginGroupBuilder};
use bevy::ecs::system::SystemState;
use bevy::prelude::*;
use bevy::render::camera::{ManualTextureView, ManualTextureViewHandle, ManualTextureViews};
use bevy::render::extract_resource::ExtractResourcePlugin;
use bevy::render::pipelined_rendering::PipelinedRenderingPlugin;
use bevy::render::renderer::{
    render_system, RenderAdapter, RenderAdapterInfo, RenderDevice, RenderInstance, RenderQueue,
};
use bevy::render::settings::RenderCreation;
use bevy::render::{Render, RenderApp, RenderPlugin, RenderSet};
use bevy::window::{PresentMode, PrimaryWindow, RawHandleWrapper, WindowMode};
use graphics::extensions::XrExtensions;
use graphics::{XrAppInfo, XrPreferdBlendMode};
use input::XrInput;
pub use openxr as xr;
use passthrough::{start_passthrough, supports_passthrough};
use resources::*;
use wgpu::Instance;
use xr::FormFactor;
use xr_init::{xr_only, XrEnableStatus};
use xr_input::controllers::XrControllerType;
use xr_input::hands::emulated::HandEmulationPlugin;
use xr_input::hands::hand_tracking::{HandTrackingData, HandTrackingPlugin};
use xr_input::OpenXrInput;

const VIEW_TYPE: xr::ViewConfigurationType = xr::ViewConfigurationType::PRIMARY_STEREO;

pub const LEFT_XR_TEXTURE_HANDLE: ManualTextureViewHandle = ManualTextureViewHandle(1208214591);
pub const RIGHT_XR_TEXTURE_HANDLE: ManualTextureViewHandle = ManualTextureViewHandle(3383858418);

/// Adds OpenXR support to an App
#[derive(Default)]
pub struct OpenXrPlugin {
    reqeusted_extensions: XrExtensions,
    prefered_blend_mode: XrPreferdBlendMode,
    app_info: XrAppInfo,
}

// #[derive(Resource)]
pub struct XrEvents(pub Vec<Box<xr::EventDataBuffer>>);

pub struct XrGraphicsData {
    device: RenderDevice,
    queue: RenderQueue,
    adapter_info: RenderAdapterInfo,
    render_adapter: RenderAdapter,
    instance: Instance,
    xr_instance: XrInstance,
    session: XrSession,
    blend_mode: XrEnvironmentBlendMode,
    resolution: XrResolution,
    format: XrFormat,
    session_running: XrSessionRunning,
    frame_waiter: XrFrameWaiter,
    swapchain: XrSwapchain,
    input: XrInput,
    views: XrViews,
    frame_state: XrFrameState,
}

impl Plugin for OpenXrPlugin {
    fn build(&self, app: &mut App) {
        let mut system_state: SystemState<Query<&RawHandleWrapper, With<PrimaryWindow>>> =
            SystemState::new(&mut app.world);
        let primary_window = system_state.get(&app.world).get_single().ok().cloned();

        #[cfg(not(target_arch = "wasm32"))]
        match graphics::initialize_xr_graphics(
            primary_window.clone(),
            self.reqeusted_extensions.clone(),
            self.prefered_blend_mode,
            self.app_info.clone(),
        ) {
            Ok(render_data) => {
                // std::thread::sleep(Duration::from_secs(5));
                debug!(
                    "Configured wgpu adapter Limits: {:#?}",
                    render_data.device.limits()
                );
                debug!(
                    "Configured wgpu adapter Features: {:#?}",
                    render_data.device.features()
                );
                app.insert_resource(render_data.xr_instance);
                app.insert_resource(render_data.session);
                app.insert_resource(render_data.blend_mode);
                app.insert_resource(render_data.resolution);
                app.insert_resource(render_data.format);
                app.insert_resource(render_data.session_running);
                app.insert_resource(render_data.frame_waiter);
                app.insert_resource(render_data.swapchain);
                app.insert_resource(render_data.input);
                app.insert_resource(render_data.views);
                app.insert_resource(render_data.frame_state);
                app.insert_resource(ActionSets(vec![]));
                app.insert_non_send_resource(XrEvents(Vec::new()));
                let render_plugin = RenderPlugin {
                    render_creation: RenderCreation::Manual(
                        render_data.device,
                        render_data.queue,
                        render_data.adapter_info,
                        render_data.render_adapter,
                        RenderInstance(Arc::new(render_data.instance)),
                    ),
                    ..default()
                };
                render_plugin.build(app);
                assert!(render_plugin.ready(app));
                render_plugin.finish(app);
                app.insert_resource(XrEnableStatus::Enabled);
            }
            Err(err) => {
                warn!("OpenXR Failed to initialize: {err}");
                app.add_plugins(RenderPlugin::default());
                app.insert_resource(XrEnableStatus::Disabled);
            }
        }
        // app.add_systems(PreUpdate, mr_test);
        #[cfg(target_arch = "wasm32")]
        {
            app.add_plugins(RenderPlugin::default());
            app.insert_resource(XrEnableStatus::Disabled);
        }
    }

    fn ready(&self, app: &App) -> bool {
        app.world
            .get_resource::<XrEnableStatus>()
            .map(|frr| *frr != XrEnableStatus::Waiting)
            .unwrap_or(true)
    }

    fn finish(&self, app: &mut App) {
        let instance = app.world.resource::<XrInstance>().clone();
        let session = app.world.resource::<XrSession>().clone();
        let swapchain = app.world.resource::<XrSwapchain>().clone();
        let resolution = app.world.resource::<XrResolution>().clone();
        let format = app.world.resource::<XrFormat>().clone();

        let hands = instance.exts().ext_hand_tracking.is_some()
            && instance
                .supports_hand_tracking(instance.system(FormFactor::HEAD_MOUNTED_DISPLAY).unwrap())
                .is_ok_and(|v| v);
        if hands {
            app.insert_resource(HandTrackingData::new(&session).unwrap());
        } else {
            app.insert_resource(DisableHandTracking::Both);
        }
        let passthrough = instance.exts().fb_passthrough.is_some()
            && supports_passthrough(
                &instance,
                instance.system(FormFactor::HEAD_MOUNTED_DISPLAY).unwrap(),
            );

        let (left, right) = swapchain.get_render_views();
        let left = ManualTextureView {
            texture_view: left.into(),
            size: *resolution,
            format: *format,
        };
        let right = ManualTextureView {
            texture_view: right.into(),
            size: *resolution,
            format: *format,
        };
        app.add_systems(PreUpdate, xr_begin_frame.run_if(xr_only()));
        let mut manual_texture_views = app.world.resource_mut::<ManualTextureViews>();
        manual_texture_views.insert(LEFT_XR_TEXTURE_HANDLE, left);
        manual_texture_views.insert(RIGHT_XR_TEXTURE_HANDLE, right);
        // drop(manual_texture_views);
        let render_app = app.sub_app_mut(RenderApp);

        if passthrough {
            info!("Passthrough!");
            let (pl, _p) = start_passthrough(&instance, &session);
            render_app.insert_resource(pl);
            info!("Inserted XrPassthroughLayer resource!");
            // app.insert_resource(p);
            // if !app.world.contains_resource::<ClearColor>() {
            // info!("ClearColor!");
            // }
        }

        render_app
            .insert_resource(XrEnableStatus::Enabled)
            .add_systems(
                Render,
                (
                    post_frame
                        .run_if(xr_only())
                        .before(render_system)
                        .after(RenderSet::ExtractCommands),
                    end_frame.run_if(xr_only()).after(render_system),
                ),
            );

        app.add_plugins((
            ExtractResourcePlugin::<XrInstance>::default(),
            ExtractResourcePlugin::<XrSession>::default(),
            ExtractResourcePlugin::<XrEnvironmentBlendMode>::default(),
            ExtractResourcePlugin::<XrResolution>::default(),
            ExtractResourcePlugin::<XrFormat>::default(),
            ExtractResourcePlugin::<XrSessionRunning>::default(),
            ExtractResourcePlugin::<XrSwapchain>::default(),
            ExtractResourcePlugin::<XrInput>::default(),
            ExtractResourcePlugin::<XrViews>::default(),
            ExtractResourcePlugin::<XrFrameState>::default(),
        ));
    }
}

#[derive(Default)]
pub struct DefaultXrPlugins {
    pub reqeusted_extensions: XrExtensions,
    pub prefered_blend_mode: XrPreferdBlendMode,
    pub app_info: XrAppInfo,
}

impl PluginGroup for DefaultXrPlugins {
    fn build(self) -> PluginGroupBuilder {
        DefaultPlugins
            .build()
            .disable::<RenderPlugin>()
            .disable::<PipelinedRenderingPlugin>()
            .add_before::<RenderPlugin, _>(OpenXrPlugin {
                prefered_blend_mode: self.prefered_blend_mode,
                reqeusted_extensions: self.reqeusted_extensions,
                app_info: self.app_info.clone(),
            })
            .add_after::<OpenXrPlugin, _>(OpenXrInput::new(XrControllerType::OculusTouch))
            .add_before::<OpenXrPlugin, _>(RenderRestartPlugin)
            .add(HandEmulationPlugin)
            .add(HandTrackingPlugin)
            .set(WindowPlugin {
                #[cfg(not(target_os = "android"))]
                primary_window: Some(Window {
                    transparent: true,
                    present_mode: PresentMode::AutoNoVsync,
                    title: self.app_info.name.clone(),
                    ..default()
                }),
                #[cfg(target_os = "android")]
                primary_window: Some(Window {
                    resizable: false,
                    mode: WindowMode::BorderlessFullscreen,
                    ..default()
                }),
                #[cfg(target_os = "android")]
                exit_condition: bevy::window::ExitCondition::DontExit,
                #[cfg(target_os = "android")]
                close_when_requested: true,
                ..default()
            })
    }
}

pub fn xr_begin_frame(
    instance: Res<XrInstance>,
    session: Res<XrSession>,
    session_running: Res<XrSessionRunning>,
    mut frame_state: ResMut<XrFrameState>,
    mut frame_waiter: ResMut<XrFrameWaiter>,
    swapchain: Res<XrSwapchain>,
    mut views: ResMut<XrViews>,
    input: Res<XrInput>,
    mut events: NonSendMut<XrEvents>,
    mut app_exit: EventWriter<AppExit>,
) {
    {
        let _span = info_span!("xr_poll_events");
        let mut new_events = Vec::new();
        loop {
            let mut evt_buf = Box::<xr::EventDataBuffer>::default();
            if let Some(event) = instance.poll_event(evt_buf.as_mut()).unwrap() {
                use xr::Event::*;
                match event {
                    SessionStateChanged(e) => {
                        // Session state change is where we can begin and end sessions, as well as
                        // find quit messages!
                        info!("entered XR state {:?}", e.state());
                        match e.state() {
                            xr::SessionState::READY => {
                                session.begin(VIEW_TYPE).unwrap();
                                session_running.store(true, std::sync::atomic::Ordering::Relaxed);
                            }
                            xr::SessionState::STOPPING => {
                                // session.end().unwrap();
                                // session_running.store(false, std::sync::atomic::Ordering::Relaxed);
                                // app_exit.send(AppExit);
                            }
                            xr::SessionState::EXITING | xr::SessionState::LOSS_PENDING => {
                                app_exit.send(AppExit);
                                return;
                            }
                            _ => {}
                        }
                    }
                    InstanceLossPending(_) => return,
                    EventsLost(e) => {
                        warn!("lost {} XR events", e.lost_event_count());
                    }
                    _ => {}
                }
                new_events.push(evt_buf);
            } else {
                break;
            }
        }

        *events = XrEvents(new_events);
    }
    {
        let _span = info_span!("xr_wait_frame").entered();
        *frame_state = match frame_waiter.wait() {
            Ok(fs) => fs.into(),
            Err(e) => {
                warn!("error: {}", e);
                return;
            }
        };
    }
    {
        let _span = info_span!("xr_begin_frame").entered();
        swapchain.begin().unwrap()
    }
    {
        let _span = info_span!("xr_locate_views").entered();
        *views = session
            .locate_views(VIEW_TYPE, frame_state.predicted_display_time, &input.stage)
            .unwrap()
            .1
            .into();
    }
}

pub fn post_frame(
    resolution: Res<XrResolution>,
    format: Res<XrFormat>,
    swapchain: Res<XrSwapchain>,
    mut manual_texture_views: ResMut<ManualTextureViews>,
) {
    {
        let _span = info_span!("xr_acquire_image").entered();
        swapchain.acquire_image().unwrap()
    }
    {
        let _span = info_span!("xr_wait_image").entered();
        swapchain.wait_image().unwrap();
    }
    {
        let _span = info_span!("xr_update_manual_texture_views").entered();
        let (left, right) = swapchain.get_render_views();
        let left = ManualTextureView {
            texture_view: left.into(),
            size: **resolution,
            format: **format,
        };
        let right = ManualTextureView {
            texture_view: right.into(),
            size: **resolution,
            format: **format,
        };
        manual_texture_views.insert(LEFT_XR_TEXTURE_HANDLE, left);
        manual_texture_views.insert(RIGHT_XR_TEXTURE_HANDLE, right);
    }
}

pub fn end_frame(
    xr_frame_state: Res<XrFrameState>,
    views: Res<XrViews>,
    input: Res<XrInput>,
    swapchain: Res<XrSwapchain>,
    resolution: Res<XrResolution>,
    environment_blend_mode: Res<XrEnvironmentBlendMode>,
    passthrough_layer: Option<Res<XrPassthroughLayer>>,
) {
    #[cfg(target_os = "android")]
    {
        let ctx = ndk_context::android_context();
        let vm = unsafe { jni::JavaVM::from_raw(ctx.vm().cast()) }.unwrap();
        let env = vm.attach_current_thread_as_daemon();
    }

    {
        let _span = info_span!("xr_release_image").entered();
        swapchain.release_image().unwrap();
    }
    {
        let _span = info_span!("xr_end_frame").entered();
        let result = swapchain.end(
            xr_frame_state.predicted_display_time,
            &views,
            &input.stage,
            **resolution,
            **environment_blend_mode,
            passthrough_layer.map(|p| p.into_inner()),
        );
        match result {
            Ok(_) => {}
            Err(e) => warn!("error: {}", e),
        }
    }
}

pub fn locate_views(
    mut views: ResMut<XrViews>,
    input: Res<XrInput>,
    session: Res<XrSession>,
    xr_frame_state: Res<XrFrameState>,
) {
    let _span = info_span!("xr_locate_views").entered();
    *views = match session.locate_views(
        VIEW_TYPE,
        xr_frame_state.predicted_display_time,
        &input.stage,
    ) {
        Ok(this) => this,
        Err(err) => {
            warn!("error: {}", err);
            return;
        }
    }
    .1
    .into();
}
