import { invoke } from '@tauri-apps/api/core'



export async function openLocationSettings(): Promise<void> {
  await invoke<void>(
    'plugin:location-settings|open_location_settings',
  )
}