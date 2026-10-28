#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_http::init())// Register the HTTP plugin
        .plugin(tauri_plugin_camera_kessdev::init())
        .plugin(tauri_plugin_location_settings::init())
        .setup(|app| {

             #[cfg(mobile)]
                app.handle()
                    .plugin(tauri_plugin_geolocation::init())?;

            if cfg!(debug_assertions) {
                app.handle()
                .plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while building tauri application");
}
