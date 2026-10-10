// Export the COM entry points under their undecorated names. Without this the
// 32-bit build exports `_DllRegisterServer@0` and regsvr32 cannot find it.
fn main() {
    println!("cargo:rerun-if-changed=exports.def");
    let msvc = std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc");
    if msvc {
        // Native separators for the *build host* (this path is read by the linker running
        // there), so this also works when cross-compiling from macOS or Linux.
        let def = std::path::Path::new(&std::env::var("CARGO_MANIFEST_DIR").unwrap()).join("exports.def");
        println!("cargo:rustc-link-arg-cdylib=/DEF:{}", def.display());
    }
}
