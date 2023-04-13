fn main() -> Result<(), Box<dyn std::error::Error>> {
    if !std::process::Command::new("rake")
        .env("MRUBY_CONFIG", "build_config.rb")
        .status()?
        .success()
    {
        panic!("failed to build mruby");
    }
    println!("cargo:rustc-link-search=native=miam2tf/vendor/mruby/build/host/lib");
    println!("cargo:rustc-link-lib=static=mruby");

    println!("cargo:rerun-if-changed=include/wrapper.h");
    println!("cargo:rerun-if-changed=src/wrapper.c");
    println!("cargo:rerun-if-changed=build_config.rb");
    println!("cargo:rerun-if-changed=mrblib/miam.rb");

    let bindings = bindgen::Builder::default()
        .clang_arg("-Ivendor/mruby/include")
        .header("include/wrapper.h")
        .blocklist_item("FP_NAN")
        .blocklist_item("FP_INFINITE")
        .blocklist_item("FP_ZERO")
        .blocklist_item("FP_SUBNORMAL")
        .blocklist_item("FP_NORMAL")
        .generate()
        .expect("unable to generate bindings");
    let out_path = std::path::PathBuf::from(std::env::var("OUT_DIR")?);
    bindings.write_to_file(out_path.join("bindings.rs"))?;
    Ok(())
}
