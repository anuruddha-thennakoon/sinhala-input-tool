import Cocoa

/// Floating candidate popup in the style of Google Input Tools: the typed text on
/// top, numbered candidates below, the highlighted row in light grey, page chevrons.
final class CandidatePanel: NSPanel {
    private let popupView = PopupView()

    init() {
        super.init(contentRect: .zero, styleMask: [.borderless, .nonactivatingPanel],
                   backing: .buffered, defer: false)
        isOpaque = false
        backgroundColor = .clear
        hasShadow = true
        level = NSWindow.Level(rawValue: Int(CGShieldingWindowLevel()))
        collectionBehavior = [.canJoinAllSpaces, .fullScreenAuxiliary, .transient]
        ignoresMouseEvents = true
        hidesOnDeactivate = false
        contentView = popupView
    }

    override var canBecomeKey: Bool { false }
    override var canBecomeMain: Bool { false }

    /// Shows `popup` just below `caret` (screen coordinates), flipping above if needed.
    func show(_ popup: Popup, below caret: NSRect) {
        popupView.popup = popup
        let size = popupView.fittingSize
        let screen = NSScreen.screens.first { $0.frame.contains(caret.origin) } ?? NSScreen.main
        let area = screen?.visibleFrame ?? NSRect(x: 0, y: 0, width: 1440, height: 900)

        var origin = NSPoint(x: caret.minX, y: caret.minY - size.height - 2)
        if origin.y < area.minY { origin.y = caret.maxY + 2 }          // no room below
        origin.x = min(max(origin.x, area.minX), area.maxX - size.width)
        origin.y = min(max(origin.y, area.minY), area.maxY - size.height)

        setFrame(NSRect(origin: origin, size: size), display: true)
        orderFrontRegardless()
    }

    func hide() { orderOut(nil) }
}

final class PopupView: NSView {
    var popup = Popup(buffer: "", rows: [], selected: 0, page: 0, pageCount: 1) {
        didSet { needsDisplay = true }
    }

    private let pad: CGFloat = 12
    private let rowHeight: CGFloat = 30
    private let headerHeight: CGFloat = 38
    private let footerHeight: CGFloat = 34

    private let headerFont = NSFont.systemFont(ofSize: 20)
    private let rowFont = NSFont.systemFont(ofSize: 18)   // system font falls back to Sinhala MN
    private let grey = NSColor(calibratedWhite: 0.93, alpha: 1)
    private let border = NSColor(calibratedWhite: 0.80, alpha: 1)
    private let caretBlue = NSColor(calibratedRed: 0.25, green: 0.6, blue: 0.95, alpha: 1)

    override var isFlipped: Bool { true }

    private var hasFooter: Bool { popup.pageCount > 1 }

    private func rowText(_ i: Int) -> String { "\(i + 1). \(popup.rows[i])" }

    private func width(of s: String, font: NSFont) -> CGFloat {
        ceil((s as NSString).size(withAttributes: [.font: font]).width)
    }

    override var fittingSize: NSSize {
        let widest = max(
            width(of: popup.buffer, font: headerFont) + 6,
            popup.rows.indices.map { width(of: rowText($0), font: rowFont) }.max() ?? 0
        )
        let w = max(150, widest + 2 * pad + 8)
        let h = headerHeight + CGFloat(popup.rows.count) * rowHeight + (hasFooter ? footerHeight : 6) + pad / 2
        return NSSize(width: w, height: h)
    }

    override func draw(_ dirtyRect: NSRect) {
        let bounds = self.bounds.insetBy(dx: 0.5, dy: 0.5)
        NSColor.white.setFill()
        border.setStroke()
        let frame = NSBezierPath(rect: bounds)
        frame.fill()
        frame.lineWidth = 1
        frame.stroke()

        // Header: the typed Latin text with a blue caret, as in the reference UI.
        let headerAttrs: [NSAttributedString.Key: Any] = [.font: headerFont, .foregroundColor: NSColor.black]
        (popup.buffer as NSString).draw(at: NSPoint(x: pad, y: 8), withAttributes: headerAttrs)
        let caretX = pad + width(of: popup.buffer, font: headerFont) + 2
        caretBlue.setFill()
        NSRect(x: caretX, y: 10, width: 2, height: 22).fill()

        // Rows.
        let rowAttrs: [NSAttributedString.Key: Any] = [.font: rowFont, .foregroundColor: NSColor.black]
        for i in popup.rows.indices {
            let y = headerHeight + CGFloat(i) * rowHeight
            if i == popup.selected {
                grey.setFill()
                NSRect(x: pad - 4, y: y, width: bounds.width - 2 * (pad - 4), height: rowHeight).fill()
            }
            (rowText(i) as NSString).draw(at: NSPoint(x: pad, y: y + 4), withAttributes: rowAttrs)
        }

        // Page chevrons.
        if hasFooter {
            let y = headerHeight + CGFloat(popup.rows.count) * rowHeight + 6
            drawChevron(up: true, at: NSRect(x: pad, y: y, width: 24, height: 22), enabled: popup.page > 0)
            drawChevron(up: false, at: NSRect(x: pad + 26, y: y, width: 24, height: 22),
                        enabled: popup.page < popup.pageCount - 1)
        }
    }

    private func drawChevron(up: Bool, at r: NSRect, enabled: Bool) {
        border.setStroke()
        let box = NSBezierPath(rect: r.insetBy(dx: 0.5, dy: 0.5))
        box.lineWidth = 1
        box.stroke()
        let color = enabled ? NSColor(calibratedWhite: 0.25, alpha: 1) : NSColor(calibratedWhite: 0.75, alpha: 1)
        color.setStroke()
        let cx = r.midX, cy = r.midY, dy: CGFloat = up ? -3 : 3
        let p = NSBezierPath()
        p.move(to: NSPoint(x: cx - 6, y: cy - dy))
        p.line(to: NSPoint(x: cx, y: cy + dy))
        p.line(to: NSPoint(x: cx + 6, y: cy - dy))
        p.lineWidth = 1.8
        p.stroke()
    }
}
