#[cfg(feature = "bindgen")]
pub mod bindings;

#[cfg(not(feature = "bindgen"))]
pub mod bindings {
    #![allow(warnings)]
    mod c_api;
    mod macros;
    mod status;
    pub use c_api::*;
    pub use macros::*;
    pub use status::*;
}

pub use bindings::*;
