//! Regenerates include/sinhala_core.h from the exported C ABI in src/ffi.rs.
fn main() {
    println!("cargo:rerun-if-changed=src/ffi.rs");
    println!("cargo:rerun-if-changed=cbindgen.toml");
    let dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let config = cbindgen::Config::from_file(format!("{dir}/cbindgen.toml")).unwrap();
    cbindgen::Builder::new()
        // Parse the one source file directly: no `cargo metadata`, which breaks under
        // cross-compilation because it inherits the target-specific RUSTFLAGS.
        .with_src(format!("{dir}/src/ffi.rs"))
        .with_config(config)
        .generate()
        .expect("cbindgen failed")
        .write_to_file(format!("{dir}/include/sinhala_core.h"));
}
