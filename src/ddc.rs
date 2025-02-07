use crate::error::*;
use core::ptr::{null_mut, NonNull};
use ddcutil_sys::bindings::*;

#[derive(Debug, Clone, Copy)]
#[allow(warnings)]
pub enum Input {
    HDMI(u8),
    DP(u8),
    TYPEC(u8),
}

impl From<Input> for u8 {
    fn from(input: Input) -> u8 {
        match input {
            Input::HDMI(1) => 0x11,
            Input::HDMI(2) => 0x12,
            Input::DP(1) => 0x0f,
            Input::DP(2) => 0x10,
            _ => todo!(),
        }
    }
}
impl TryFrom<u8> for Input {
    type Error = DDCError;
    fn try_from(value: u8) -> std::result::Result<Self, Self::Error> {
        Ok(match value {
            0x11 => Self::HDMI(1),
            0x12 => Self::HDMI(2),
            0x0f => Self::DP(1),
            0x10 => Self::DP(2),
            _ => Err(DDCError::new(DdcutilErrorKind::Other))?,
        })
    }
}

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

#[derive(Debug)]
pub struct DisplayInfo<'info> {
    info: &'info DDCA_Display_Info,
}

impl DisplayInfo<'_> {
    pub fn open(&self) -> Result<Display> {
        Display::open(self)
    }

    pub fn io_path(&self) -> IOPath {
        self.info.path.into()
    }

    pub fn model(&self) -> &str {
        unsafe { core::ffi::CStr::from_ptr(self.info.model_name.as_ptr()) }
            .to_str()
            .unwrap()
    }
}

#[derive(Debug)]
pub struct Display {
    handle: DDCA_Display_Handle,
}

impl Display {
    const BACKLIGHT: u8 = 0x10;
    const INPUT: u8 = 0x60;
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

    pub fn input(&self) -> Result<Input> {
        let mut out = DDCA_Non_Table_Vcp_Value {
            mh: 0,
            ml: 0,
            sh: 0,
            sl: 0,
        };
        let rc = unsafe { ddca_get_non_table_vcp_value(self.handle, Self::INPUT, &mut out) };
        LibDDCUtilError::from_rc(rc)?;
        Ok(match (out.ml, out.sl) {
            (_, 0x0f) => Input::DP(1),   // DP-1
            (_, 0x10) => Input::DP(2),   // DP-2
            (_, 0x11) => Input::HDMI(1), // HDMI-1
            (_, 0x12) => Input::HDMI(2), // HDMI-2
            _ => todo!(),
        })
    }
    pub fn set_input(&self, input: Input) -> Result<()> {
        let value: u8 = input.into();
        let rc = unsafe { ddca_set_non_table_vcp_value(self.handle, Self::INPUT, 0, value) };
        LibDDCUtilError::from_rc(rc)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct Backlight {
    pub current: u16,
    pub max: u16,
}

#[test]
fn test_input() {
    let list = DisplayList::probe(true).unwrap();
    for dinfo in list.iter() {
        tracing::info!("Found display: {}", dinfo.model());
        let display = dinfo.open().unwrap();
        dbg!(dinfo);
        dbg!(&display);
        let input = display.input().unwrap();
        dbg!(input);
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum IOPath {
    I2C(i32),
    Usb(i32),
}

impl core::fmt::Display for IOPath {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::I2C(bus) => write!(f, "/dev/i2c-{}", bus),
            Self::Usb(dev) => write!(f, "/dev/hiddev{}", dev),
        }
    }
}

impl From<DDCA_IO_Path> for IOPath {
    fn from(path: DDCA_IO_Path) -> Self {
        let discriminant = path.io_mode;
        if discriminant == DDCA_IO_Mode_DDCA_IO_I2C {
            unsafe { Self::I2C(path.path.i2c_busno) }
        } else if discriminant == DDCA_IO_Mode_DDCA_IO_USB {
            unsafe { Self::Usb(path.path.hiddev_devno) }
        } else {
            unreachable!("DDCUTIL returned an unknown IOPath");
        }
    }
}
