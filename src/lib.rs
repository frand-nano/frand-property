mod property;

mod system;
#[cfg(feature = "slint")]
mod slint;

#[cfg(feature = "slint")]
pub use frand_property_macro::slint_model;

pub use arraystring;

pub use self::{
    property::*,
    system::*,
    slint::*,
};