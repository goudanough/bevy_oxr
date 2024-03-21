use std::ptr::null;

use openxr::SystemId;

use crate::{
    resources::{XrInstance, XrPassthroughLayer},
    XrSession,
};
use openxr as xr;
use xr::{
    sys::{
        PassthroughCreateInfoFB, PassthroughFB, PassthroughLayerCreateInfoFB, PassthroughLayerFB,
    },
    PassthroughFlagsFB, PassthroughLayerPurposeFB,
};

pub fn start_passthrough(
    instance: &XrInstance,
    session: &XrSession,
) -> (XrPassthroughLayer, PassthroughFB) {
    let vtable = instance.exts().fb_passthrough.unwrap();

    // Configuration for creating the passthrough feature
    let passthrough_create_info = PassthroughCreateInfoFB {
        ty: PassthroughCreateInfoFB::TYPE,
        next: null(),
        flags: PassthroughFlagsFB::IS_RUNNING_AT_CREATION,
    };

    let mut passthrough_feature = openxr::sys::PassthroughFB::NULL;
    let result = unsafe {
        (vtable.create_passthrough)(
            session.as_raw(),
            &passthrough_create_info,
            &mut passthrough_feature,
        )
    };

    if result != openxr::sys::Result::SUCCESS {
        panic!("Failed to start passthough layer:\n{result:?}");
    }

    let passthrough = PassthroughFB::NULL;

    let passthrough_layer_info = PassthroughLayerCreateInfoFB {
        ty: PassthroughLayerCreateInfoFB::TYPE,
        next: null(),
        passthrough,
        flags: PassthroughFlagsFB::IS_RUNNING_AT_CREATION,
        purpose: PassthroughLayerPurposeFB::RECONSTRUCTION,
    };

    let mut passthrough_layer_fb = PassthroughLayerFB::NULL;
    let result = unsafe {
        (vtable.create_passthrough_layer)(
            session.as_raw(),
            &passthrough_layer_info,
            &mut passthrough_layer_fb,
        )
    };

    if result != openxr::sys::Result::SUCCESS {
        panic!("Failed to create a passthough layer:\n{result:?}");
    }

    (
        XrPassthroughLayer::new(passthrough_layer_fb),
        passthrough_feature,
    )
}

pub fn supports_passthrough(_a: &XrInstance, _b: SystemId) -> bool {
    true
}
