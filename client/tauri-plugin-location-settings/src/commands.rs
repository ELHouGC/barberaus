use tauri::{AppHandle, command, Runtime};

#[cfg(mobile)]
use crate::LocationSettingsExt;

#[command]
pub(crate) async fn open_location_settings<R: Runtime>(
    app: AppHandle<R>,
) -> std::result::Result<(), String> {
    #[cfg(mobile)]
    {
        app.location_settings()
            .open_location_settings()
            .map_err(|error| error.to_string())
    }

    #[cfg(desktop)]
    {
        let _ = app;

        Err(
            "Opening device settings is only supported on mobile."
                .to_string(),
        )
    }
}