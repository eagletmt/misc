fn main() {
    let xlib = x11_dl::xlib::Xlib::open().expect("Unable to load xlib");
    let xft = x11_dl::xft::Xft::open().expect("Unable to load xft");

    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        let desc_cstr =
            std::ffi::CString::new(args[1].clone()).expect("Unable to allocate CString");

        let dpy = unsafe { (xlib.XOpenDisplay)(std::ptr::null()) };
        let screen = unsafe { (xlib.XDefaultScreenOfDisplay)(dpy) };
        let screen_number = unsafe { (xlib.XScreenNumberOfScreen)(screen) };
        let font = unsafe { (xft.XftFontOpenName)(dpy, screen_number, desc_cstr.as_ptr()) };
        let height = unsafe { *font }.height;
        println!("{}", height);
        unsafe { (xft.XftFontClose)(dpy, font) };
        unsafe { (xlib.XCloseDisplay)(dpy) };
    } else {
        eprintln!("Usage: {} PATTERN", args[0]);
    }
}
