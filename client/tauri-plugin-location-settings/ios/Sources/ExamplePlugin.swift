
import Tauri
import UIKit

class ExamplePlugin: Plugin {
    @objc public func openLocationSettings(_ invoke: Invoke) throws {
        guard let url = URL(string: UIApplication.openSettingsURLString) else {
            invoke.reject("Unable to open Barberaus app settings.")
            return
        }

        DispatchQueue.main.async {
            UIApplication.shared.open(url, options: [:]) { opened in
                if opened {
                    invoke.resolve()
                } else {
                    invoke.reject("Unable to open Barberaus app settings.")
                }
            }
        }
    }
}

@_cdecl("init_plugin_location_settings")
func initPlugin() -> Plugin {
    ExamplePlugin()
}