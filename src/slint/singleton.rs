use slint;

#[cfg(feature = "slint")]
pub trait SlintSingleton: Sized + slint::ComponentHandle {
    fn clone_singleton() -> slint::Weak<Self>;
}
