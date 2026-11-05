# Tauri Plugin location-settings

A Tauri v2 plugin that opens platform-specific location settings.

## Supported Platforms

| Platform | Behavior |
|---|---|
| Android | Opens device-wide Location Services settings |
| iOS | Opens the application's settings page |
| Desktop | Not supported |

## Installation

Add the Rust plugin as a local dependency in your Tauri application.

Register the plugin:

```rust
.plugin(tauri_plugin_location_settings::init())


Grant the frontend capability:
"location-settings:default"

Frontend Usage
import { openLocationSettings } from "tauri-plugin-location-settings-api";
await openLocationSettings();