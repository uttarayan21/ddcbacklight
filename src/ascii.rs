
pub trait AsAscii<'lrstr> {
    fn as_ascii(&self) -> &str;
}

impl<'l> AsAscii<'l> for *const i8 {
    fn as_ascii(&self) -> &str {
        const MAX_STRLEN: usize = 100_usize;
        // This is risky since we don't own the value
        if self.is_null() {
            ""
        } else {
            let mut len = 0;
            while len < MAX_STRLEN {
                unsafe {
                    if *self.add(len) == 0 {
                        break;
                    }
                }
                len += 1;
            }
            unsafe {
                std::str::from_utf8_unchecked(std::slice::from_raw_parts(*self as *const u8, len))
            }
        }
    }
}

impl<'lrstr> AsAscii<'lrstr> for &[i8] {
    fn as_ascii(&self) -> &str {
        std::str::from_utf8(
            &unsafe { std::mem::transmute::<&[i8], &[u8]>(self) }
                [..self.iter().position(|&i| i == 0).unwrap_or_default()],
        )
        .map(|ascii| ascii.trim_matches('\0'))
        .unwrap_or_default()
    }
}

impl<'lrstr, const LEN: usize> AsAscii<'lrstr> for [i8; LEN] {
    fn as_ascii(&self) -> &str {
        std::str::from_utf8(
            &unsafe { std::mem::transmute::<&[i8; LEN], &[u8; LEN]>(self) }
                [..self.iter().position(|&i| i == 0).unwrap_or_default()],
        )
        .map(|ascii| ascii.trim_matches('\0'))
        .unwrap_or_default()
    }
}
