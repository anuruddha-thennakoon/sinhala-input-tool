// Renders the candidate popup for a typed word to a PNG, for visual checks without a host app.
// usage: render_popup <latin> <out.png>
import Cocoa

let word = CommandLine.arguments[1]
let core = SinhalaCore()
word.forEach { core.push($0) }

_ = NSApplication.shared
let view = PopupView()
view.popup = core.popup()
let size = view.fittingSize
view.frame = NSRect(origin: .zero, size: size)

let scale: CGFloat = 2
guard let rep = NSBitmapImageRep(
    bitmapDataPlanes: nil, pixelsWide: Int(size.width * scale), pixelsHigh: Int(size.height * scale),
    bitsPerSample: 8, samplesPerPixel: 4, hasAlpha: true, isPlanar: false,
    colorSpaceName: .deviceRGB, bytesPerRow: 0, bitsPerPixel: 0) else { exit(1) }
rep.size = size
view.cacheDisplay(in: view.bounds, to: rep)
try rep.representation(using: .png, properties: [:])!.write(to: URL(fileURLWithPath: CommandLine.arguments[2]))
print("rendered \(Int(size.width))x\(Int(size.height)) pt")
