// Registers and enables the Sinhala input source for the current user.
// usage: swift register.swift /path/to/SinhalaInput.app
import Carbon

let url = URL(fileURLWithPath: CommandLine.arguments[1]) as CFURL
let status = TISRegisterInputSource(url)
print("TISRegisterInputSource: \(status == noErr ? "ok" : "error \(status)")")

let filter = [kTISPropertyBundleID as String: "com.itrowin.inputmethod.Sinhala"] as CFDictionary
if let list = TISCreateInputSourceList(filter, true)?.takeRetainedValue() as? [TISInputSource] {
    for source in list {
        let r = TISEnableInputSource(source)
        print("enable: \(r == noErr ? "ok" : "error \(r)")")
    }
    if list.isEmpty { print("input source not found yet; log out and back in, then add it in System Settings") }
}
