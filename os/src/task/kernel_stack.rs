use crate::mm::address::VirtAddr;
use crate::config::{KERNEL_STACK_SIZE, PAGE_SIZE, TRAMPOLINE};
use crate::mm::memory_set::{KERNEL_SPACE, MapPermission};
use crate::task::pid::PidHandle;

pub struct KernelStack {
    pid: usize,
}

impl KernelStack {
    pub fn new(pid_handle: &PidHandle) -> Self {
        let pid = pid_handle.0;
        let (kernel_stack_bottom, kernel_stack_top) = kernel_stack_position(pid);
        KERNEL_SPACE.exclusive_access().insert_framed_areas(
            kernel_stack_bottom.into(),
            kernel_stack_top.into(),
            MapPermission::R | MapPermission::W,
        );
        Self { pid: pid_handle.0 }
    }
    pub fn top(&self) -> VirtAddr {
        let (_, kernel_stack_top) = kernel_stack_position(self.pid);
        kernel_stack_top.into()
    }
    #[allow(unused)]
    pub fn push_on_top<T>(&self, value: T) -> *mut T
        where
            T: Sized,
    {
        let kernel_stack_top = self.get_top();
        let ptr_mut = (kernel_stack_top - core::mem::size_of::<T>()) as *mut T;
        unsafe {
            *ptr_mut = value;
        }
        ptr_mut
    }
    pub fn get_top(&self) -> usize {
        let (_, kernel_stack_top) = kernel_stack_position(self.pid);
        kernel_stack_top
    }
}

impl Drop for KernelStack {
    fn drop(&mut self) {
        let (kernel_stack_bottom, _) = kernel_stack_position(self.pid);
        let kernel_stack_bottom_va: VirtAddr = kernel_stack_bottom.into();
        KERNEL_SPACE
            .exclusive_access()
            .remove_areas(kernel_stack_bottom_va.into());
    }
}

pub fn kernel_stack_position(pid: usize) -> (usize, usize) {
    let top = TRAMPOLINE - pid * (KERNEL_STACK_SIZE + PAGE_SIZE);
    let bottom = top - KERNEL_STACK_SIZE;
    (bottom, top)
}
