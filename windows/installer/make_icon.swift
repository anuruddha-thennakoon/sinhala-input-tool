// Renders PNGs of the input-method icon (white "සි" on a blue rounded square).
// usage: swift make_icon.swift <out-dir>   -> icon-16.png, icon-32.png, icon-48.png, icon-256.png
// windows/installer/make_ico.py packs them into sinhala.ico.
import Cocoa

for size in [16, 32, 48, 256] {
    let px = CGFloat(size)
    guard let rep = NSBitmapImageRep(
        bitmapDataPlanes: nil, pixelsWide: size, pixelsHigh: size, bitsPerSample: 8,
        samplesPerPixel: 4, hasAlpha: true, isPlanar: false, colorSpaceName: .deviceRGB,
        bytesPerRow: 0, bitsPerPixel: 0) else { exit(1) }
    NSGraphicsContext.saveGraphicsState()
    NSGraphicsContext.current = NSGraphicsContext(bitmapImageRep: rep)
    NSColor(calibratedRed: 0x2E / 255, green: 0x7A / 255, blue: 0xC0 / 255, alpha: 1).setFill()
    NSBezierPath(roundedRect: NSRect(x: 0, y: 0, width: px, height: px), xRadius: px * 0.18, yRadius: px * 0.18).fill()
    let attrs: [NSAttributedString.Key: Any] = [
        .font: NSFont.systemFont(ofSize: px * 0.62, weight: .semibold),
        .foregroundColor: NSColor.white,
    ]
    let text = "සි" as NSString
    let s = text.size(withAttributes: attrs)
    text.draw(at: NSPoint(x: (px - s.width) / 2, y: (px - s.height) / 2), withAttributes: attrs)
    NSGraphicsContext.restoreGraphicsState()
    let url = URL(fileURLWithPath: CommandLine.arguments[1]).appendingPathComponent("icon-\(size).png")
    try rep.representation(using: .png, properties: [:])!.write(to: url)
}
