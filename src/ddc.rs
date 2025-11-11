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
    list: NonNull<DDCA_Display_Ref>,
    len: usize,
    // handle: NonNull<DDCA_Display_Handle>,
}

impl DisplayList {
    pub fn probe(unsupported: bool) -> Result<Self> {
        tracing::info!("Check for monitors using ddca_get_displays()");
        let mut drefs: *mut DDCA_Display_Ref = null_mut();
        let rc = unsafe { ddca_get_display_refs(unsupported, &mut drefs) };
        LibDDCUtilError::from_rc(rc)?;
        let dlist_count = {
            if drefs.is_null() {
                0
            } else {
                let mut count = 0;
                let mut current = drefs;
                while !(unsafe { *current }).is_null() {
                    count += 1;
                    unsafe {
                        current = current.offset(1);
                    }
                }
                count
            }
        };

        Ok(Self {
            list: NonNull::new(drefs)
                .ok_or_else(|| DDCError::new(DdcutilErrorKind::UnknownHandle))?,
            len: dlist_count,
        })
    }

    pub fn get(&self, index: usize) -> Result<DisplayInfo<'_>> {
        if index < self.len {
            let mut info2: *mut DDCA_Display_Info2 = null_mut();
            let dref: *mut DDCA_Display_Ref = unsafe { self.list.as_ptr().offset(index as isize) };
            let rc = unsafe { ddca_get_display_info2(*dref, &mut info2) };
            LibDDCUtilError::from_rc(rc)?;
            let info2 = unsafe { &*info2 };
            Ok(DisplayInfo { info: info2 })
        } else {
            Err(DDCError::new(DdcutilErrorKind::OutOfRange))
        }
    }

    pub fn iter(&self) -> DisplayListIter<'_> {
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
            let out = self
                .list
                .get(self.index)
                .inspect_err(|f| {
                    tracing::error!(
                        "Error getting display info at index {}: {:?}",
                        self.index,
                        f
                    );
                })
                .ok();
            self.index += 1;
            out
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct DisplayInfo<'info> {
    info: &'info DDCA_Display_Info2,
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
            .expect("Invalid UTF-8 in model name")
    }

    pub fn drm(&self) -> String {
        let out = String::from_utf8_lossy(
            self.info
                .drm_card_connector
                .iter()
                .map(|&c| c as u8)
                .take_while(|&c| c != 0)
                .collect::<Vec<u8>>()
                .as_slice(),
        )
        .to_string();
        out
        // unsafe { core::ffi::CStr::from_ptr(self.info.drm_card_connector.as_ptr()) }
        //     .to_str()
        //     .ok()
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

pub fn version() -> semver::Version {
    let version = unsafe { ddca_ddcutil_version() };
    semver::Version::new(
        version.major as u64,
        version.minor as u64,
        version.micro as u64,
    )
}
