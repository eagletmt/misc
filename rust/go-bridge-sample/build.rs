fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());

    println!("cargo:rerun-if-changed=bridge.go");
    println!("cargo:rustc-link-lib=static=go-bridge-sample");
    println!("cargo:rustc-link-search=native={}", out_dir.display());
    let status = std::process::Command::new("go")
        .arg("build")
        .arg("-buildmode=c-archive")
        .arg(format!(
            "-o={}",
            out_dir.join("libgo-bridge-sample.a").display()
        ))
        .status()?;
    if !status.success() {
        panic!("Failed  to build libgo-bridge-sample.a");
    }

    let bindings = bindgen::Builder::default()
        .header(format!(
            "{}",
            out_dir.join("libgo-bridge-sample.h").display()
        ))
        .generate()
        .expect("Failed to generate bindings.rs from libgo-bridge-sample.h");
    bindings.write_to_file(out_dir.join("bindings.rs"))?;
    Ok(())
}
