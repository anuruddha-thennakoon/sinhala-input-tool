// Renders the menu-bar input source icon (black glyph, transparent background).
// usage: swift make_icon.swift out.tiff
import Cocoa

let size = NSSize(width: 32, height: 32)
let image = NSImage(size: size)
image.lockFocus()
let attrs: [NSAttributedString.Key: Any] = [
    .font: NSFont.systemFont(ofSize: 24, weight: .semibold),
    .foregroundColor: NSColor.black,
]
let text = "සි" as NSString
let s = text.size(withAttributes: attrs)
text.draw(at: NSPoint(x: (size.width - s.width) / 2, y: (size.height - s.height) / 2), withAttributes: attrs)
image.unlockFocus()
guard let tiff = image.tiffRepresentation else { exit(1) }
try tiff.write(to: URL(fileURLWithPath: CommandLine.arguments[1]))
