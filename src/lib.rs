mod property;

mod system;
#[cfg(feature = "slint")]
mod slint;

#[cfg(feature = "slint")]
pub use frand_property_macro::slint_model;

pub use frand_property_macro::model;

pub use arraystring;

pub use self::{
    property::*,
    system::*,
    slint::*,
};

pub mod model;
pub use model::{Model, ModelList};