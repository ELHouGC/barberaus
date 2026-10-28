use serde::de::DeserializeOwned;
use tauri::{
  plugin::{PluginApi, PluginHandle},
  AppHandle, Runtime,
};

use crate::models::*;

#[cfg(target_os = "ios")]
tauri::ios_plugin_binding!(init_plugin_location_settings);

// initializes the Kotlin or Swift plugin classes
pub fn init<R: Runtime, C: DeserializeOwned>(
  _app: &AppHandle<R>,
  api: PluginApi<R, C>,
) -> crate::Result<LocationSettings<R>> {
  #[cfg(target_os = "android")]
  let handle = api.register_android_plugin("com.plugin.locationsettings", "ExamplePlugin")?;
  #[cfg(target_os = "ios")]
  let handle = api.register_ios_plugin(init_plugin_location_settings)?;
  Ok(LocationSettings(handle))
}

/// Access to the location-settings APIs.
pub struct LocationSettings<R: Runtime>(PluginHandle<R>);

impl<R: Runtime> LocationSettings<R> {
  pub fn ping(&self, payload: PingRequest) -> crate::Result<PingResponse> {
    self
      .0
      .run_mobile_plugin("ping", payload)
      .map_err(Into::into)
  }
  pub fn open_location_settings(&self) -> crate::Result<()> {
    self.0
        .run_mobile_plugin("openLocationSettings", ())
        .map_err(Into::into)
  }
}
