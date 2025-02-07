mod ascii;
mod ddc;
mod error;
use core::marker::PhantomData;
pub use ddc::*;
use error::Result;

/// The main entry point for the library.
/// This contains the main struct that will be used to interact with all the monitors
pub struct DDC {}

/// The ddc driver that will be used to interact with the monitors
pub enum DDCDriver {
    Windows(WindowsDDC),
    Linux(LinuxDDC),
}

/// Uses windows native functions to interact
pub struct WindowsDDC {}

/// Uses ddcutil to interact with the monitors
pub struct LinuxDDC {}

/// A display identifier
/// Placeholder
pub struct DisplayIdent {
    __marker: PhantomData<()>,
}

/// Placeholder
pub struct VCPFeature {
    __marker: PhantomData<()>,
}

pub trait DDCDriverTrait {
    fn probe(&self) -> Result<DisplayList>;
    fn get_vcp(&self, display: &DisplayIdent, vcp: u8) -> Result<VCPFeature>;
    fn set_vcp(&self, display: &DisplayIdent, vcp: u8) -> Result<()>;
}
