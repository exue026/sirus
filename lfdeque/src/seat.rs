use std::{mem::{self}, ptr::NonNull, ptr};

#[derive(Debug)]
struct SeatInner<T> {
    data: T
}

#[derive(Debug)]
pub struct Seat<T> {
    ptr: NonNull<SeatInner<T>>,
}

impl<T> Seat<T> {
    pub fn new(data: T) -> Self {
        let ptr = Box::new(SeatInner {data});
        Self::from_inner(Box::leak(ptr).into())
    }

    fn from_inner(ptr: NonNull<SeatInner<T>>) -> Self {
        Self { ptr }
    }

    pub fn take(this: Self) -> T {
        unsafe {
            let elem = ptr::read(&this.ptr.as_ref().data);

            mem::forget(this);

            elem
        }        
    }
}

impl<T> Clone for Seat<T> {
    fn clone(&self) -> Self {
        Self::from_inner(self.ptr)
    }
}

unsafe impl<T> Send for Seat<T> where T: Send {}
unsafe impl<T> Sync for Seat<T> where T: Send {}
