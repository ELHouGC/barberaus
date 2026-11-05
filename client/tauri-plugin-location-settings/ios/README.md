# Tauri Plugin: Location Settings

A Tauri v2 plugin for opening platform-specific location settings.

## Supported Platforms

- Android: Opens device-wide Location Services settings.
- iOS: Opens the application's settings page.

## Usage

```typescript
import { openLocationSettings } from "tauri-plugin-location-settings-api";

await openLocationSettings();
