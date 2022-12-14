// use crate::libs::rc::CancerCell;
use crate::libs::rc::CancerCell;
// use lazy_static::__Deref;
// use crate::traits::observer::Observable;

use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

pub trait Observable {
    type Target;
    fn get_observers(&mut self) -> &mut Vec<Box<dyn FnMut(&Self::Target, &Self::Target) -> ()>>;
    fn watch(&mut self, f: impl FnMut(&Self::Target, &Self::Target) -> () + 'static) {
        self.get_observers().push(Box::new(f));
    }
    fn notify(&mut self, old_value: &Self::Target, new_value: &Self::Target) {
        self.get_observers().iter_mut().for_each(|f| {
            f(old_value, new_value);
        })
    }
}

pub struct Global<T> {
    old_value: Option<T>,
    value: T,
    observers: Vec<Box<dyn FnMut(&T, &T) -> ()>>,
}
impl<T> Global<T> {
    pub fn new(value: T) -> Self {
        Self {
            old_value: None,
            value,
            observers: vec![],
        }
    }
    #[allow(unused)]
    pub fn change(&mut self, value: T)
    where
        T: Clone,
    {
        self.old_value = Some(self.value.clone());
        self.value = value;
        unsafe { self.notify() }
    }
    pub fn get_value(&self) -> &T {
        &self.value
    }
    pub unsafe fn notify(&mut self) {
        let old_value = self.old_value.as_ref().unwrap() as *const T;
        let new_value = &self.value as *const T;
        Observable::notify(self, &*old_value, &*new_value);
    }
}
impl<T> Observable for Global<T> {
    type Target = T;
    fn get_observers(&mut self) -> &mut Vec<Box<dyn FnMut(&Self::Target, &Self::Target) -> ()>> {
        &mut self.observers
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
