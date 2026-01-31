#[cfg(feature = "slint")]
pub use frand_property_macro::slint_model;

pub use frand_property_macro::model;

mod property;
mod model;

#[cfg(feature = "slint")]
pub mod slint;

mod stream;
mod receiver_group;

pub use self::{
    property::*,
    model::*,
    stream::*,
    receiver_group::*,
};
