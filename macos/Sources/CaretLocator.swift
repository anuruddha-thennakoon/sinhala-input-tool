import Cocoa
import InputMethodKit

/// Opt-in debug log: create `~/Library/Application Support/SinhalaInput/enable-log`
/// (an empty file) and entries go to `debug.log` in the same folder.
enum DebugLog {
    private static var dir: URL {
        FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask)[0]
            .appendingPathComponent("SinhalaInput")
    }

    static func write(_ message: @autoclosure () -> String) {
        let marker = dir.appendingPathComponent("enable-log")
        guard FileManager.default.fileExists(atPath: marker.path) else { return }
        let line = "[\(ISO8601DateFormatter().string(from: Date()))] \(message())\n"
        let file = dir.appendingPathComponent("debug.log")
        if let handle = try? FileHandle(forWritingTo: file) {
            handle.seekToEndOfFile()
            handle.write(Data(line.utf8))
            try? handle.close()
        } else {
            try? line.write(to: file, atomically: true, encoding: .utf8)
        }
    }
}

/// Finds where the text cursor is on screen, for placing the popup.
///
/// Host apps answer `attributes(forCharacterIndex:lineHeightRectangle:)` inconsistently:
/// Word may return an empty rectangle, a stale one, or only answer for some indexes. So try
/// several indexes, accept only a plausible on-screen rectangle, and remember the last good
/// one instead of falling back to the mouse (which put the popup mid-page).
struct CaretLocator {
    private(set) var lastGood: NSRect?

    /// Rectangle in screen coordinates, or nil if the app gave nothing usable.
    mutating func locate(_ client: IMKTextInput) -> NSRect? {
        let marked = client.markedRange()
        let selected = client.selectedRange()
        var indexes: [Int] = []
        if marked.location != NSNotFound {
            indexes += [marked.location + marked.length, marked.location]
        }
        if selected.location != NSNotFound { indexes.append(selected.location) }
        indexes.append(0)

        for index in indexes where index >= 0 {
            var rect = NSRect.zero
            _ = client.attributes(forCharacterIndex: index, lineHeightRectangle: &rect)
            DebugLog.write("caret index=\(index) marked=\(marked) selected=\(selected) rect=\(rect)")
            if Self.plausible(rect) {
                lastGood = rect
                return rect
            }
        }
        DebugLog.write("no plausible caret rect; lastGood=\(String(describing: lastGood))")
        return nil
    }

    /// Non-empty, finite, a sane line height, and on a connected screen.
    static func plausible(_ r: NSRect) -> Bool {
        guard r.origin.x.isFinite, r.origin.y.isFinite, r.width.isFinite, r.height.isFinite else { return false }
        guard r != .zero, r.height > 0, r.height < 400 else { return false }
        return NSScreen.screens.contains { $0.frame.insetBy(dx: -50, dy: -50).intersects(r) }
    }

    /// Best effort when nothing usable came back: last good position, else the mouse.
    func fallback() -> NSRect {
        if let lastGood { return lastGood }
        let m = NSEvent.mouseLocation
        return NSRect(x: m.x, y: m.y, width: 1, height: 18)
    }
}
