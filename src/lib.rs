mod property;

#[cfg(feature = "slint")]
pub use frand_property_macro::slint_model;

pub use self::{
    property::*,
};