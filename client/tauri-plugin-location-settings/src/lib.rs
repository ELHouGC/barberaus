use tauri::{
    plugin::{Builder, TauriPlugin},
    Runtime,
};

#[cfg(mobile)]
use tauri::Manager;

mod commands;

#[cfg(mobile)]
mod error;
#[cfg(mobile)]
mod mobile;

#[cfg(mobile)]
pub use error::{Error, Result};

#[cfg(mobile)]
use mobile::LocationSettings;

/// Extensions to access the location-settings APIs.
#[cfg(mobile)]
pub trait LocationSettingsExt<R: Runtime> {
    fn location_settings(&self) -> &LocationSettings<R>;
}

#[cfg(mobile)]
impl<R: Runtime, T: Manager<R>> LocationSettingsExt<R> for T {
    fn location_settings(&self) -> &LocationSettings<R> {
        self.state::<LocationSettings<R>>().inner()
    }
}

/// Initializes the plugin.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("location-settings")
        .invoke_handler(tauri::generate_handler![
            commands::open_location_settings
        ])
        .setup(|app, api| {
            #[cfg(mobile)]
            app.manage(mobile::init(app, api)?);

            #[cfg(not(mobile))]
            let _ = (app, api);

            Ok(())
        })
        .build()
}