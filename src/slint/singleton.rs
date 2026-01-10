use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use slint;

static SINGLETON_INSTANCES: OnceLock<Mutex<HashMap<TypeId, Box<dyn Any + Send + Sync>>>> = OnceLock::new();

pub trait SlintSingleton: Sized + slint::ComponentHandle + 'static {
    fn init_singleton(&self) {
        let map = SINGLETON_INSTANCES
            .get_or_init(|| Mutex::new(HashMap::new()));

        let mut map = map.lock().unwrap();

        map.insert(TypeId::of::<Self>(), Box::new(self.as_weak()));
    }

    fn clone_singleton() -> slint::Weak<Self> {
        let map = SINGLETON_INSTANCES.get();

        let map = map.as_ref()
            .expect("Singletons not initialized. Call init_singleton() first.");

        let map = map.lock().unwrap();

        let weak = map.get(&TypeId::of::<Self>())
            .expect("Singleton not initialized. Call init_singleton() first.");

        weak.downcast_ref::<slint::Weak<Self>>().expect("Type mismatch in singleton store").clone()
    }
}

impl<T: slint::ComponentHandle + 'static> SlintSingleton for T {}
