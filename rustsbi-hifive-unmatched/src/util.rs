use core::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
};

/// Use only amo instructions on mutex; no lr/sc instruction is used
pub struct AmoMutex<T: ?Sized> {
    lock: UnsafeCell<u8>,
    data: UnsafeCell<T>,
}

pub struct AmoMutexGuard<'a, T: ?Sized> {
    lock: *mut u8,
    data: &'a mut T,
}

impl<T> AmoMutex<T> {
    /// Create a new AmoMutex
    pub const fn new(data: T) -> Self {
        AmoMutex {
            data: UnsafeCell::new(data),
            lock: UnsafeCell::new(0),
        }
    }
    /// Locks the mutex and returns a guard that permits access to the inner data.
    pub fn lock(&self) -> AmoMutexGuard<T> {
        unsafe {
            core::arch::asm!(
                "li     {one}, 1",
                "1: lw  {tmp}, ({lock})", // check if lock is held
                // "call   {relax}", // spin loop hint
                "bnez   {tmp}, 1b", // retry if held
                "amoswap.w.aq {tmp}, {one}, ({lock})", // attempt to acquire lock
                "bnez   {tmp}, 1b", // retry if held
                lock = in(reg) self.lock.get(),
                tmp = out(reg) _,
                one = out(reg) _,
                // relax = sym pause,
                options(nostack)
            );
        }
        AmoMutexGuard {
            lock: self.lock.get(),
            data: unsafe { &mut *self.data.get() },
        }
    }
    // pub unsafe fn force_unlock(&self) {
    //     *self.lock.get() = 0
    // }
}

unsafe impl<T: ?Sized + Send> Sync for AmoMutex<T> {}
unsafe impl<T: ?Sized + Send> Send for AmoMutex<T> {}

impl<'a, T: ?Sized> Deref for AmoMutexGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.data
    }
}

impl<'a, T: ?Sized> DerefMut for AmoMutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.data
    }
}

impl<'a, T: ?Sized> Drop for AmoMutexGuard<'a, T> {
    /// The dropping of the mutex guard will release the lock it was created from.
    fn drop(&mut self) {
        unsafe {
            core::arch::asm!(
                "amoswap.w.rl x0, x0, ({lock})", // release lock by storing 0
                lock = in(reg) self.lock,
            );
        }
    }
}
