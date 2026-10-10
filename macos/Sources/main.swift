import Cocoa
import InputMethodKit

// Keeps the connection alive for the lifetime of the process.
let connectionName = Bundle.main.infoDictionary?["InputMethodConnectionName"] as? String
let server = IMKServer(name: connectionName, bundleIdentifier: Bundle.main.bundleIdentifier)

let app = NSApplication.shared
app.setActivationPolicy(.accessory)
app.run()
