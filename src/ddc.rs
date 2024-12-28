use crate::ascii::AsAscii;
use crate::error::*;
use core::ptr::{null_mut, NonNull};
use ddcutil_sys::bindings::*;

pub struct DisplayList {
    list: NonNull<DDCA_Display_Info_List>,
    len: usize,
    // handle: NonNull<DDCA_Display_Handle>,
}

impl DisplayList {
    pub fn probe(unsupported: bool) -> Result<Self> {
        tracing::info!("Check for monitors using ddca_get_displays()");
        let mut dlist: *mut DDCA_Display_Info_List = null_mut();
        let rc = unsafe { ddca_get_display_info_list2(unsupported, &mut dlist) };
        assert!(!dlist.is_null());
        LibDDCUtilError::from_rc(rc)?;
        let dlist = unsafe { &mut *dlist };
        if dlist.ct == 0 {
            tracing::info!("No DDC capable displays found");
            return Err(DDCError::new(DdcutilErrorKind::NoDisplays));
        } else {
            tracing::trace!("Found {} DDC capable displays", dlist.ct);
        }

        Ok(Self {
            // handle: NonNull::new(&mut dh)
            //     .ok_or_else(|| DDCError::new(DdcutilErrorKind::UnknownHandle))?,
            list: NonNull::new(dlist)
                .ok_or_else(|| DDCError::new(DdcutilErrorKind::UnknownHandle))?,
            len: dlist.ct as usize,
        })
    }
    pub fn iter(&self) -> DisplayListIter {
        DisplayListIter {
            list: self,
            index: 0,
        }
    }
}

pub struct DisplayListIter<'dl> {
    list: &'dl DisplayList,
    index: usize,
}

impl<'i> Iterator for DisplayListIter<'i> {
    type Item = DisplayInfo<'i>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.list.len {
            let dinf: &DDCA_Display_Info =
                unsafe { &self.list.list.as_ref().info.as_slice(self.list.len)[self.index] };
            self.index += 1;
            Some(DisplayInfo { info: dinf })
        } else {
            None
        }
    }
}

pub struct DisplayInfo<'info> {
    info: &'info DDCA_Display_Info,
}

impl DisplayInfo<'_> {
    pub fn open(&self) -> Result<Display> {
        Display::open(self)
    }

    pub fn model(&self) -> &str {
        unsafe { core::ffi::CStr::from_ptr(self.info.model_name.as_ptr()) }
            .to_str()
            .unwrap()
    }
}

pub struct Display {
    handle: DDCA_Display_Handle,
}

impl Display {
    const BACKLIGHT: u8 = 0x10;
    pub fn open(info: &DisplayInfo) -> Result<Self> {
        let dref = info.info.dref;
        let mut dh = null_mut();
        let rc = unsafe { ddca_open_display2(dref, true, &mut dh) };
        LibDDCUtilError::from_rc(rc)?;
        Ok(Self { handle: dh })
    }

    pub fn backlight_set(&self, value: u16) -> Result<()> {
        if value > 100 {
            return Err(DDCError::new(DdcutilErrorKind::OutOfRange));
        }
        let [hi_byte, lo_byte] = value.to_be_bytes();
        tracing::trace!("Setting backlight to {} ({} {})", value, hi_byte, lo_byte);
        let rc =
            unsafe { ddca_set_non_table_vcp_value(self.handle, Self::BACKLIGHT, hi_byte, lo_byte) };
        LibDDCUtilError::from_rc(rc)?;
        Ok(())
    }

    pub fn backlight_get(&self) -> Result<Backlight> {
        let mut out = DDCA_Non_Table_Vcp_Value {
            mh: 0,
            ml: 0,
            sh: 0,
            sl: 0,
        };
        let rc = unsafe { ddca_get_non_table_vcp_value(self.handle, Self::BACKLIGHT, &mut out) };
        LibDDCUtilError::from_rc(rc)?;
        Ok(Backlight {
            max: u16::from_be_bytes([out.mh, out.ml]),
            current: u16::from_be_bytes([out.sh, out.sl]),
        })
    }
}

#[derive(Debug)]
pub struct Backlight {
    pub current: u16,
    pub max: u16,
}
