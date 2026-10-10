import Foundation

/// What the popup draws. Mirrors `PopupView` in the Rust core.
struct Popup {
    var buffer: String
    var rows: [String]      // candidates on this page, then the raw Latin text
    var selected: Int
    var page: Int
    var pageCount: Int
}

/// Swift wrapper over the C ABI in core/include/sinhala_core.h.
final class SinhalaCore {
    private let handle: OpaquePointer?

    init() { handle = sinhala_session_new() }
    deinit { sinhala_session_free(handle) }

    private func take(_ p: UnsafeMutablePointer<CChar>?) -> String? {
        guard let p = p else { return nil }
        defer { sinhala_string_free(p) }
        return String(cString: p)
    }

    var isComposing: Bool { sinhala_is_composing(handle) }
    var buffer: String { take(sinhala_buffer(handle)) ?? "" }

    func push(_ c: Character) {
        for scalar in c.unicodeScalars { _ = sinhala_push_char(handle, scalar.value) }
    }
    func backspace() -> Bool { sinhala_backspace(handle) }
    func move(_ delta: Int) { sinhala_move_selection(handle, Int32(delta)) }
    func turnPage(_ delta: Int) { sinhala_turn_page(handle, Int32(delta)) }
    func commit() -> String? { take(sinhala_commit(handle)) }
    func pick(_ n: Int) -> String? { take(sinhala_select_number(handle, UInt32(n))) }
    func cancel() { sinhala_cancel(handle) }

    func popup() -> Popup {
        let count = Int(sinhala_row_count(handle))
        let rows = (0..<count).compactMap { take(sinhala_row(handle, UInt32($0))) }
        return Popup(buffer: buffer, rows: rows, selected: Int(sinhala_selected_row(handle)),
                     page: Int(sinhala_page(handle)), pageCount: Int(sinhala_page_count(handle)))
    }
}
