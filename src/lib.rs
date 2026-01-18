#[cfg(feature = "slint")]
pub use frand_property_macro::slint_model;

pub use frand_property_macro::model;

pub use arraystring;

mod property;
mod model;

#[cfg(feature = "slint")]
mod slint;
mod stream;
mod receiver_group;


pub use self::{
    property::*,
    slint::*,
    model::*,
    stream::*,
    receiver_group::*,
};
