pub mod extensions;
mod vulkan;

use crate::XrGraphicsData;
use bevy::window::RawHandleWrapper;

use openxr as xr;

use self::extensions::XrExtensions;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum XrPreferdBlendMode {
    Opaque,
    Additive,
    AlphaBlend,
}
impl Default for XrPreferdBlendMode {
    fn default() -> Self {
        Self::Opaque
    }
}

#[derive(Clone, Debug)]
pub struct XrAppInfo {
    pub name: String,
}
impl Default for XrAppInfo {
    fn default() -> Self {
        Self {
            name: "Ambient".into(),
        }
    }
}

pub fn initialize_xr_graphics(
    window: Option<RawHandleWrapper>,
    reqeusted_extensions: XrExtensions,
    prefered_blend_mode: XrPreferdBlendMode,
    app_info: XrAppInfo,
) -> anyhow::Result<XrGraphicsData> {
    vulkan::initialize_xr_graphics(window, reqeusted_extensions, prefered_blend_mode, app_info)
}

pub fn xr_entry() -> anyhow::Result<xr::Entry> {
    #[cfg(windows)]
    return Ok(xr::Entry::linked());
    #[cfg(not(windows))]
    return unsafe { xr::Entry::load() }.map_err(|e| anyhow::anyhow!(e));
}
