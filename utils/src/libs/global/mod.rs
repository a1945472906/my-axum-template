// use crate::libs::rc::CancerCell;
use crate::libs::rc::CancerCell;
// use lazy_static::__Deref;
// use crate::traits::observer::Observable;

use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

pub trait Observable {
    type Target;
    fn get_observers(&mut self) -> &mut Vec<Box<dyn FnMut(&Self::Target) -> ()>>;
    fn watch(&mut self, f: impl FnMut(&Self::Target) -> () + 'static) {
        self.get_observers().push(Box::new(f));
    }
    fn notify(&mut self, new_value: &Self::Target) {
        self.get_observers().iter_mut().for_each(|f| {
            f(new_value);
        })
    }
}

pub struct Global<T> {
    value: T,
    observers: Vec<Box<dyn FnMut(&T) -> ()>>,
}
impl<T> Observable for Global<T> {
    type Target = T;
    fn get_observers(&mut self) -> &mut Vec<Box<dyn FnMut(&Self::Target) -> ()>> {
        &mut self.observers
    }
}
impl<T> Global<T> {
    pub fn new(value: T) -> Self {
        Self {
            value,
            observers: vec![],
        }
    }
    #[allow(unused)]
    pub fn change(&mut self, value: T)
    where
        T: Clone,
    {
        self.value = value;
    }
    pub fn get_value(&self) -> &T {
        &self.value
    }
    pub unsafe fn notify(&mut self) {
        let ptr = &self.value as *const T;
        Observable::notify(self, &*ptr);
    }
}
impl<T> Deref for Global<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
impl<T> DerefMut for Global<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}
unsafe impl<T> Send for Global<T> {}
unsafe impl<T> Sync for Global<T> {}

static ENV: Lazy<CancerCell<Global<HashMap<String, String>>>> = Lazy::new(|| {
    let mut env = HashMap::new();
    dotenv::dotenv().expect("Can not found .env file");
    for (k, v) in std::env::vars() {
        env.insert(k, v);
    }
    CancerCell::new(Global::new(env))
});

pub fn get_global_env() -> &'static mut Global<HashMap<String, String>> {
    ENV.get_mut()
}
