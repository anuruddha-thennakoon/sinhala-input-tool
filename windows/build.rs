// Export the COM entry points under their undecorated names. Without this the
// 32-bit build exports `_DllRegisterServer@0` and regsvr32 cannot find it.
fn main() {
    println!("cargo:rerun-if-changed=exports.def");
    let msvc = std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc");
    if msvc {
        let dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        println!("cargo:rustc-link-arg-cdylib=/DEF:{dir}\\exports.def");
    }
}
