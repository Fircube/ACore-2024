use core::cell::{RefCell, RefMut};

// Allows us to safely use mutable global variables on uni-processor.
// Provides internal variability and runtime borrowing checks
pub struct UPSafeCell<T> {
    // inner data
    inner: RefCell<T>,
}

unsafe impl<T> Sync for UPSafeCell<T> {}

impl<T> UPSafeCell<T> {
    // User is responsible to guarantee that inner struct is only used in uni-processor.
    pub unsafe fn new(value: T) -> Self {
        Self {
            inner: RefCell::new(value),
        }
    }
    // Exclusive access inner data in UPSafeCell. Panic if the data has been borrowed.
    // Not allow multiple read operations to exist at the same time.
    pub fn exclusive_access(&self) -> RefMut<'_, T> {
        self.inner.borrow_mut()
    }
}