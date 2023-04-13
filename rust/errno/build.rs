#[allow(clippy::type_complexity)]
static MACROS: once_cell::sync::OnceCell<std::sync::Mutex<std::cell::RefCell<Vec<(String, i32)>>>> =
    once_cell::sync::OnceCell::new();

#[derive(Debug, Default)]
struct MacroCollector {}

impl bindgen::callbacks::ParseCallbacks for MacroCollector {
    fn int_macro(&self, name: &str, value: i64) -> Option<bindgen::callbacks::IntKind> {
        if name.starts_with('E') {
            MACROS
                .get()
                .unwrap()
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
    MACROS.set(Default::default()).unwrap();

    bindgen::builder()
        .header_contents("wrapper.h", "#include <errno.h>")
        .parse_callbacks(Box::<MacroCollector>::default())
        .generate()
        .expect("unable to collect macros");

    let out_dir = std::env::var("OUT_DIR")?;
    let mut file = std::fs::File::create(std::path::Path::new(&out_dir).join("errno.rs"))?;
    writeln!(file, "pub const ERROR_NUMBERS: &[(&str, i32, &str)] = &[")?;
    for (name, value) in MACROS.get().unwrap().lock()?.borrow().iter() {
        let message = unsafe { std::ffi::CStr::from_ptr(libc::strerror(*value)).to_str()? };
        writeln!(file, "(\"{}\", {}, \"{}\"),", name, value, message)?;
    }
    writeln!(file, "];")?;
    Ok(())
}
