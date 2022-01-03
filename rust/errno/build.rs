lazy_static::lazy_static! {
    static ref MACROS: std::sync::Mutex<std::cell::RefCell<Vec<(String, i32)>>> = Default::default();
}

#[derive(Debug, Default)]
struct MacroCollector {}

impl bindgen::callbacks::ParseCallbacks for MacroCollector {
    fn int_macro(&self, name: &str, value: i64) -> Option<bindgen::callbacks::IntKind> {
        if name.starts_with('E') {
            MACROS
                .lock()
                .unwrap()
                .borrow_mut()
                .push((name.to_owned(), value as i32));
        }
        None
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    use std::io::Write as _;

    bindgen::builder()
        .header_contents("wrapper.h", "#include <errno.h>")
        .parse_callbacks(Box::new(MacroCollector::default()))
        .generate()
        .expect("unable to collect macros");

    let out_dir = std::env::var("OUT_DIR")?;
    let mut file = std::fs::File::create(std::path::Path::new(&out_dir).join("errno.rs"))?;
    writeln!(file, "pub const ERROR_NUMBERS: &[(&str, i32, &str)] = &[")?;
    for (name, value) in MACROS.lock()?.borrow().iter() {
        let message = unsafe { std::ffi::CStr::from_ptr(libc::strerror(*value)).to_str()? };
        writeln!(file, "(\"{}\", {}, \"{}\"),", name, value, message)?;
    }
    writeln!(file, "];")?;
    Ok(())
}
