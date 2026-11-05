
package com.plugin.locationsettings

import android.app.Activity
import android.content.Intent
import android.provider.Settings
import app.tauri.annotation.Command
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.Plugin


@TauriPlugin
class LocationSettingsPlugin(
    private val activity: Activity,
) : Plugin(activity) {

    @Command
    fun openLocationSettings(invoke: Invoke) {
        try {
            val intent = Intent(Settings.ACTION_LOCATION_SOURCE_SETTINGS)
            activity.startActivity(intent)
            invoke.resolve()
        } catch (error: Exception) {
            invoke.reject("Unable to open Location settings.")
        }
    }
}
