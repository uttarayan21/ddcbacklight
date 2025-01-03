pub trait AsAscii<'lrstr> {
    fn as_ascii(&self) -> &str;
}

impl AsAscii<'_> for *const i8 {
    fn as_ascii(&self) -> &str {
        unsafe {
            core::ffi::CStr::from_ptr(*self)
                .to_str()
                .unwrap_or_default()
        }
    }
}
