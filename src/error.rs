use crate::ascii::AsAscii;
use core::fmt::Display;
use ddcutil_sys::bindings::{ddca_rc_desc, ddca_rc_name, DDCA_Status};
use error_stack::Report;
use thiserror::Error;

pub type Result<T, E = DDCError> = core::result::Result<T, E>;

#[derive(Debug)]
pub struct DDCError {
    #[allow(dead_code)]
    kind: Report<DdcutilErrorKind>,
}
impl DDCError {
    #[track_caller]
    pub(crate) fn new(no_displays: DdcutilErrorKind) -> DDCError {
        DDCError {
            kind: Report::from(no_displays),
        }
    }
}

#[derive(Debug, Error)]
pub enum DdcutilErrorKind {
    #[error(transparent)]
    LibDDCUtilError(#[from] LibDDCUtilError),
    #[error("No displays found")]
    NoDisplays,
    #[error("Unable to initialize the display handle")]
    UnknownHandle,
    #[error("Out of Range")]
    OutOfRange,
}

#[derive(Debug, Error)]
pub struct LibDDCUtilError(DDCA_Status);

impl Display for LibDDCUtilError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}, {}, {:#x}",
            unsafe { ddca_rc_name(self.0) }.as_ascii(),
            unsafe { ddca_rc_desc(self.0) }.as_ascii(),
            self.0
        )
    }
}

impl From<DDCA_Status> for LibDDCUtilError {
    #[track_caller]
    fn from(status: DDCA_Status) -> Self {
        Self(status)
    }
}

impl LibDDCUtilError {
    #[track_caller]
    pub fn from_rc(status: DDCA_Status) -> Result<()> {
        if status == 0 {
            Ok(())
        } else {
            Err(DDCError::new(DdcutilErrorKind::LibDDCUtilError(
                status.into(),
            )))
        }
    }
}
