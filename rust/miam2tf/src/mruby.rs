use std::io::Read as _;

pub struct MRuby {
    mrb: *mut crate::mruby_c::mrb_state,
}

pub struct Value<'a> {
    mruby: &'a MRuby,
    inner: crate::mruby_c::mrb_value,
}

impl Default for MRuby {
    fn default() -> Self {
        let mrb = unsafe {
            let mrb = crate::mruby_c::mrb_open();
            add_missing_methods(mrb);
            mrb
        };
        Self { mrb }
    }
}

impl Drop for MRuby {
    fn drop(&mut self) {
        unsafe {
            crate::mruby_c::mrb_close(self.mrb);
        }
    }
}

impl MRuby {
    pub fn load<P>(&self, path: P) -> Result<(), anyhow::Error>
    where
        P: AsRef<std::path::Path>,
    {
        let path_cstr = std::ffi::CString::new(format!("{}", path.as_ref().display()))?;
        let mut file = std::fs::File::open(path)?;
        let mut code = Vec::new();
        file.read_to_end(&mut code)?;

        unsafe {
            let ctx = crate::mruby_c::mrbc_context_new(self.mrb);
            crate::mruby_c::mrbc_filename(self.mrb, ctx, path_cstr.as_ptr());
            crate::mruby_c::mrb_load_nstring_cxt(
                self.mrb,
                code.as_ptr() as *const i8,
                code.len(),
                ctx,
            );
            crate::mruby_c::mrbc_context_free(self.mrb, ctx);
            if !(*self.mrb).exc.is_null() {
                crate::mruby_c::mrb_print_error(self.mrb);
                return Err(anyhow::anyhow!("MRuby error"));
            }
        }
        Ok(())
    }

    pub fn instance_variable_get<'a>(&'a self, name: &'static str) -> Value<'a> {
        let value = unsafe {
            crate::mruby_c::mrb_obj_iv_get(
                self.mrb,
                (*self.mrb).top_self,
                crate::mruby_c::mrb_intern_static(self.mrb, name.as_ptr() as *const i8, name.len()),
            )
        };
        Value {
            mruby: self,
            inner: value,
        }
    }
}

unsafe fn add_missing_methods(mrb: *mut crate::mruby_c::mrb_state) {
    let dir_class =
        crate::mruby_c::mrb_define_class(mrb, c"Dir".as_ptr(), (*mrb).object_class);
    crate::mruby_c::mrb_define_class_method(
        mrb,
        dir_class,
        c"glob".as_ptr(),
        Some(mrb_dir_glob),
        mrb_args_req(1),
    );
    crate::mruby_c::mrb_define_singleton_method(
        mrb,
        (*mrb).top_self,
        c"require".as_ptr(),
        Some(mrb_require),
        mrb_args_req(1),
    );
}

fn mrb_args_req(n: i64) -> crate::mruby_c::mrb_aspec {
    unsafe { crate::mruby_c::wrapper_mrb_args_req(n) }
}

fn rarray_len(ary: crate::mruby_c::mrb_value) -> i64 {
    unsafe { crate::mruby_c::wrapper_rarray_len(ary) }
}

fn mrb_nil_p(o: crate::mruby_c::mrb_value) -> bool {
    unsafe { crate::mruby_c::wrapper_mrb_nil_p(o) != 0 }
}

fn mrb_nil_value() -> crate::mruby_c::mrb_value {
    unsafe { crate::mruby_c::wrapper_mrb_nil_value() }
}

fn unwrap_or_raise<T, E>(mrb: *mut crate::mruby_c::mrb_state, r: Result<T, E>) -> T
where
    E: std::error::Error,
{
    match r {
        Ok(v) => v,
        Err(e) => {
            let msg = std::ffi::CString::new(format!("{:?}", e)).unwrap();
            unsafe {
                crate::mruby_c::mrb_raise(
                    mrb,
                    crate::mruby_c::wrapper_e_runtime_error(mrb),
                    msg.as_ptr(),
                )
            };
            unreachable!();
        }
    }
}

extern "C" fn mrb_dir_glob(
    mrb: *mut crate::mruby_c::mrb_state,
    _self: crate::mruby_c::mrb_value,
) -> crate::mruby_c::mrb_value {
    let mut block = mrb_nil_value();
    let pat = unsafe {
        let mut val = mrb_nil_value();
        crate::mruby_c::mrb_get_args(mrb, c"S&".as_ptr(), &mut val, &mut block);
        to_rust_string(mrb, val)
    };

    if mrb_nil_p(block) {
        let entries = unsafe { crate::mruby_c::mrb_ary_new(mrb) };
        for entry in unwrap_or_raise(mrb, glob::glob(&pat)) {
            let path = unwrap_or_raise(mrb, entry);
            let path_str = format!("{}", path.display());
            unsafe {
                let path_value = crate::mruby_c::mrb_str_new(
                    mrb,
                    path_str.as_ptr() as *const i8,
                    path_str.len(),
                );
                crate::mruby_c::mrb_ary_push(mrb, entries, path_value);
            };
        }
        entries
    } else {
        for entry in unwrap_or_raise(mrb, glob::glob(&pat)) {
            let path = unwrap_or_raise(mrb, entry);
            let path_str = format!("{}", path.display());
            unsafe {
                let path_value = crate::mruby_c::mrb_str_new(
                    mrb,
                    path_str.as_ptr() as *const i8,
                    path_str.len(),
                );
                let method_name = "call";
                let method_id = crate::mruby_c::mrb_intern_static(
                    mrb,
                    method_name.as_ptr() as *const i8,
                    method_name.len(),
                );
                let argv = [path_value];
                crate::mruby_c::mrb_funcall_argv(
                    mrb,
                    block,
                    method_id,
                    argv.len() as i64,
                    argv.as_ptr(),
                );
            };
        }
        mrb_nil_value()
    }
}

extern "C" fn mrb_require(
    mrb: *mut crate::mruby_c::mrb_state,
    _self: crate::mruby_c::mrb_value,
) -> crate::mruby_c::mrb_value {
    let path = unsafe {
        let mut val = mrb_nil_value();
        crate::mruby_c::mrb_get_args(mrb, c"S".as_ptr(), &mut val);
        to_rust_string(mrb, val)
    };
    let mut file = unwrap_or_raise(mrb, std::fs::File::open(&path));
    let mut code = Vec::new();
    unwrap_or_raise(mrb, file.read_to_end(&mut code));
    unsafe {
        let path_cstr = unwrap_or_raise(mrb, std::ffi::CString::new(path.as_bytes()));
        let ctx = crate::mruby_c::mrbc_context_new(mrb);
        crate::mruby_c::mrbc_filename(mrb, ctx, path_cstr.as_ptr());
        crate::mruby_c::mrb_load_nstring_cxt(mrb, code.as_ptr() as *const i8, code.len(), ctx);
        crate::mruby_c::mrbc_context_free(mrb, ctx);
    }
    mrb_nil_value()
}

fn to_rust_string(mrb: *mut crate::mruby_c::mrb_state, s: crate::mruby_c::mrb_value) -> String {
    unsafe {
        std::ffi::CStr::from_ptr(crate::mruby_c::mrb_str_to_cstr(mrb, s))
            .to_string_lossy()
            .into_owned()
    }
}

pub struct ValueIter<'a> {
    idx: i64,
    len: i64,
    ary: crate::mruby_c::mrb_value,
    mruby: &'a MRuby,
}
impl<'a> Iterator for ValueIter<'a> {
    type Item = Value<'a>;

    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        if self.idx < self.len {
            let val = unsafe { crate::mruby_c::mrb_ary_entry(self.ary, self.idx) };
            self.idx += 1;
            Some(Value {
                mruby: self.mruby,
                inner: val,
            })
        } else {
            None
        }
    }
}

impl<'a> Value<'a> {
    pub fn read_attribute(&self, name: &'static str) -> Value<'a> {
        let value = unsafe {
            let meth = crate::mruby_c::mrb_intern_static(
                self.mruby.mrb,
                name.as_ptr() as *const i8,
                name.len(),
            );
            crate::mruby_c::mrb_funcall_argv(self.mruby.mrb, self.inner, meth, 0, std::ptr::null())
        };
        Value {
            mruby: self.mruby,
            inner: value,
        }
    }

    pub fn is_nil(&self) -> bool {
        mrb_nil_p(self.inner)
    }

    #[allow(clippy::inherent_to_string)]
    pub fn to_string(&self) -> String {
        to_rust_string(self.mruby.mrb, self.inner)
    }

    pub fn to_string_opt(&self) -> Option<String> {
        if self.is_nil() {
            None
        } else {
            Some(self.to_string())
        }
    }

    pub fn to_i64(&self) -> i64 {
        unsafe { crate::mruby_c::wrapper_mrb_integer(self.inner) }
    }

    pub fn to_i64_opt(&self) -> Option<i64> {
        if self.is_nil() {
            None
        } else {
            Some(self.to_i64())
        }
    }

    pub fn iter(&self) -> ValueIter<'a> {
        ValueIter {
            idx: 0,
            len: rarray_len(self.inner),
            ary: self.inner,
            mruby: self.mruby,
        }
    }
}
