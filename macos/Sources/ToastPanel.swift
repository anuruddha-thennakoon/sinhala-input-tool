import Cocoa

/// Brief confirmation near the caret ("Sinhala typing: ON") so a toggle is never silent.
final class ToastPanel: NSPanel {
    private let label = NSTextField(labelWithString: "")
    private var hideWork: DispatchWorkItem?

    init() {
        super.init(contentRect: .zero, styleMask: [.borderless, .nonactivatingPanel],
                   backing: .buffered, defer: false)
        isOpaque = false
        backgroundColor = .clear
        hasShadow = true
        level = NSWindow.Level(rawValue: Int(CGShieldingWindowLevel()))
        collectionBehavior = [.canJoinAllSpaces, .fullScreenAuxiliary, .transient]
        ignoresMouseEvents = true
        label.font = .systemFont(ofSize: 15, weight: .medium)
        label.textColor = .white
        label.alignment = .center
        let box = NSView()
        box.wantsLayer = true
        box.layer?.backgroundColor = NSColor(calibratedWhite: 0.15, alpha: 0.92).cgColor
        box.layer?.cornerRadius = 8
        box.addSubview(label)
        contentView = box
    }

    override var canBecomeKey: Bool { false }

    func show(_ text: String, near caret: NSRect, for seconds: TimeInterval = 1.2) {
        label.stringValue = text
        label.sizeToFit()
        let size = NSSize(width: label.frame.width + 28, height: label.frame.height + 16)
        label.setFrameOrigin(NSPoint(x: 14, y: 8))
        let screen = NSScreen.screens.first { $0.frame.contains(caret.origin) } ?? NSScreen.main
        let area = screen?.visibleFrame ?? NSRect(x: 0, y: 0, width: 1440, height: 900)
        var origin = NSPoint(x: caret.minX, y: caret.minY - size.height - 4)
        if origin.y < area.minY { origin.y = caret.maxY + 4 }
        origin.x = min(max(origin.x, area.minX), area.maxX - size.width)
        origin.y = min(max(origin.y, area.minY), area.maxY - size.height)
        setFrame(NSRect(origin: origin, size: size), display: true)
        orderFrontRegardless()

        hideWork?.cancel()
        let work = DispatchWorkItem { [weak self] in self?.orderOut(nil) }
        hideWork = work
        DispatchQueue.main.asyncAfter(deadline: .now() + seconds, execute: work)
    }
}
