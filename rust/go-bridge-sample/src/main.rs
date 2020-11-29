fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = "input.cue";
    let code = std::fs::read_to_string(path)?;
    let exported = cue_export(path, &code)?;
    println!("{}", exported);
    Ok(())
}

fn cue_export(filename: &str, code: &str) -> Result<String, String> {
    let filename_cstr = std::ffi::CString::new(filename).unwrap();
    let code_cstr = std::ffi::CString::new(code).unwrap();
    let mut e = 0;
    let result = unsafe {
        let ptr = go_bridge_sample::cue_export(filename_cstr.as_ptr(), code_cstr.as_ptr(), &mut e);
        let r = std::ffi::CStr::from_ptr(ptr).to_string_lossy().into_owned();
        libc::free(ptr as *mut libc::c_void);
        r
    };
    if e == 0 {
        Ok(result)
    } else {
        Err(result)
    }
}
