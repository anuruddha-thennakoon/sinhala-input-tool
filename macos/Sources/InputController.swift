import Cocoa
import InputMethodKit

/// One instance per client text field. Owns a typing session in the Rust core and
/// translates between key events, the session, the host app's text, and the popup.
@objc(SinhalaInputController)
final class SinhalaInputController: IMKInputController {
    private let core = SinhalaCore()
    private static let panel = CandidatePanel()
    private static let toast = ToastPanel()
    private let noRange = NSRange(location: NSNotFound, length: NSNotFound)

    override func recognizedEvents(_ sender: Any!) -> Int {
        Int(NSEvent.EventTypeMask.keyDown.rawValue)
    }

    override func handle(_ event: NSEvent!, client sender: Any!) -> Bool {
        guard let event = event, event.type == .keyDown,
              let client = sender as? IMKTextInput else { return false }

        let flags = event.modifierFlags.intersection(.deviceIndependentFlagsMask)
        let action = KeyRouter.route(
            keyCode: event.keyCode, chars: event.characters ?? "",
            command: flags.contains(.command), control: flags.contains(.control),
            option: flags.contains(.option), shift: flags.contains(.shift),
            composing: core.isComposing, enabled: AppState.shared.enabled)

        switch action {
        case .passThrough:
            return false
        case .type(let c):
            core.push(c)
            refresh(client)
        case .moveSelection(let d):
            core.move(d)
            refresh(client)
        case .turnPage(let d):
            core.turnPage(d)
            refresh(client)
        case .backspace:
            if core.backspace() { refresh(client) } else { return false }
        case .pickNumber(let n):
            if let text = core.pick(n) { insert(text, client) } else { refresh(client) }
        case .commit:
            commitSelection(client)
        case .commitAndSpace:
            commitSelection(client, suffix: " ")
        case .cancel:
            core.cancel()
            insert("", client)   // clears the marked text
        case .commitThenPassThrough:
            commitSelection(client)
            return false
        case .toggleEnabled:
            if core.isComposing { commitSelection(client) }
            toggleEnabled(client)
        }
        return true
    }

    // MARK: - Input menu (the Sinhala icon in the menu bar)

    override func menu() -> NSMenu! {
        let menu = NSMenu(title: "Sinhala Input")
        let toggle = NSMenuItem(title: "Sinhala typing  (Ctrl+Shift+Space)",
                                action: #selector(toggleFromMenu(_:)), keyEquivalent: "")
        toggle.target = self
        toggle.state = AppState.shared.enabled ? .on : .off
        menu.addItem(toggle)
        menu.addItem(.separator())
        let about = NSMenuItem(title: "About Sinhala Input…", action: #selector(showAbout(_:)), keyEquivalent: "")
        about.target = self
        menu.addItem(about)
        return menu
    }

    @objc private func toggleFromMenu(_ sender: Any?) {
        if let client = client() {
            if core.isComposing { commitSelection(client) }
            toggleEnabled(client)
        } else {
            AppState.shared.toggle()
        }
    }

    @objc private func showAbout(_ sender: Any?) {
        NSApp.activate(ignoringOtherApps: true)
        let alert = NSAlert()
        alert.messageText = "Sinhala Input"
        alert.informativeText = aboutText()
        alert.addButton(withTitle: "OK")
        alert.runModal()
    }

    private func toggleEnabled(_ client: IMKTextInput) {
        let on = AppState.shared.toggle()
        Self.toast.show(on ? "Sinhala typing: ON" : "Sinhala typing: OFF", near: caretRect(client))
    }

    // The app is taking focus away or asked us to finish: don't lose what was typed.
    override func commitComposition(_ sender: Any!) {
        guard let client = sender as? IMKTextInput else { return }
        commitSelection(client)
    }

    override func activateServer(_ sender: Any!) {
        AppState.shared.reload()   // picks up a change made while another app had focus
        super.activateServer(sender)
    }

    override func deactivateServer(_ sender: Any!) {
        if let client = sender as? IMKTextInput { commitSelection(client) }
        Self.panel.hide()
        super.deactivateServer(sender)
    }

    // MARK: - Helpers

    private func commitSelection(_ client: IMKTextInput, suffix: String = "") {
        let text = core.commit() ?? ""
        insert(text + suffix, client)
    }

    /// Replaces the marked text with `text` and hides the popup.
    private func insert(_ text: String, _ client: IMKTextInput) {
        client.insertText(text, replacementRange: noRange)
        Self.panel.hide()
    }

    /// Shows the Latin buffer as underlined marked text and the popup below the caret.
    private func refresh(_ client: IMKTextInput) {
        guard core.isComposing else {
            insert("", client)
            return
        }
        let buffer = core.buffer
        client.setMarkedText(
            NSAttributedString(string: buffer, attributes: [
                .underlineStyle: NSUnderlineStyle.single.rawValue]),
            selectionRange: NSRange(location: buffer.utf16.count, length: 0),
            replacementRange: noRange)
        Self.panel.show(core.popup(), below: caretRect(client))
    }

    private func caretRect(_ client: IMKTextInput) -> NSRect {
        var rect = NSRect.zero
        let marked = client.markedRange()
        let index = marked.location == NSNotFound ? 0 : max(0, marked.location)
        _ = client.attributes(forCharacterIndex: index, lineHeightRectangle: &rect)
        if rect == .zero || rect.origin.x.isNaN {
            let m = NSEvent.mouseLocation          // client gave nothing usable
            rect = NSRect(x: m.x, y: m.y, width: 1, height: 18)
        }
        return rect
    }
}
