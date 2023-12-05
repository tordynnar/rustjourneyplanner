pub fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=Cargo.lock");

    let root_dir: std::path::PathBuf = std::env::var("CARGO_MANIFEST_DIR").unwrap().into();
    let generated_dir = root_dir.join("generated");

    leptonic_theme::generate(generated_dir.join("leptonic"));
    println!("cargo:warning=theme written");
}