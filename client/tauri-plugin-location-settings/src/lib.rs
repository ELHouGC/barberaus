use tauri::{
  plugin::{Builder, TauriPlugin},
  Manager, Runtime,
};

pub use models::*;

#[cfg(desktop)]
mod desktop;
#[cfg(mobile)]
mod mobile;

mod commands;
mod error;
mod models;

pub use error::{Error, Result};

#[cfg(desktop)]
use desktop::LocationSettings;
#[cfg(mobile)]
use mobile::LocationSettings;

/// Extensions to [`tauri::App`], [`tauri::AppHandle`] and [`tauri::Window`] to access the location-settings APIs.
pub trait LocationSettingsExt<R: Runtime> {
  fn location_settings(&self) -> &LocationSettings<R>;
}

impl<R: Runtime, T: Manager<R>> crate::LocationSettingsExt<R> for T {
  fn location_settings(&self) -> &LocationSettings<R> {
    self.state::<LocationSettings<R>>().inner()
  }
}

/// Initializes the plugin.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
  Builder::new("location-settings")
    .invoke_handler(tauri::generate_handler![commands::open_location_settings],)
    .setup(|app, api| {
      #[cfg(mobile)]
      let location_settings = mobile::init(app, api)?;
      #[cfg(desktop)]
      let location_settings = desktop::init(app, api)?;
      app.manage(location_settings);
      Ok(())
    })
    .build()
}
