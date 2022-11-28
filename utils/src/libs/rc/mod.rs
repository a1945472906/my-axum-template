use std::ops::{Deref, DerefMut};

pub struct CancerCell<T> {
    value: T,
}
#[allow(unused)]
impl<T> CancerCell<T> {
    pub const fn new(value: T) -> Self {
        Self { value }
    }
    pub fn get(&self) -> &T {
        &self.value
    }
    pub fn get_mut(&self) -> &mut T {
        unsafe { &mut *(&self.value as *const T as *mut T) }
    }
    pub fn get_ptr(&self) -> Ptr<T> {
        Ptr(&self.value as *const T as *mut T)
    }
    pub fn get_raw(&self) -> *const T {
        &self.value as *const T
    }
    pub fn get_mut_raw(&self) -> *mut T {
        &self.value as *const T as *mut T
    }
}

// #[derive(Clone, Copy)]
pub struct Ptr<T>(*mut T);
unsafe impl<T> Sync for Ptr<T> {}
unsafe impl<T> Send for Ptr<T> {}

impl<T> Deref for Ptr<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0 }
    }
}
impl<T> DerefMut for Ptr<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.0 }
    }
}
impl<T> Copy for Ptr<T> {}

impl<T> Clone for Ptr<T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}
