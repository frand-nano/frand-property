use slint;

#[cfg(feature = "slint")]
pub trait SlintSingleton: Sized + slint::ComponentHandle {
    fn get_singleton_instance() -> slint::Weak<Self>;
}
