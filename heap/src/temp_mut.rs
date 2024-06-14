use core::cell::RefCell;

// a fake mutex as learned in UPSafeCell
pub struct TempMut<T> {
    pub inner: RefCell<T>,
}

unsafe impl<T> Sync for TempMut<T> {}

impl<T> TempMut<T> {
    pub const fn new(inner: T) -> Self {
        Self {
            inner: RefCell::new(inner),
        }
    }
}