// use std::{mem::{self}, ptr::NonNull, ptr, sync::atomic::{AtomicUsize, Ordering}};

// #[derive(Debug)]
// struct SeatInner<T> {
//     data: Option<T>
// }

// impl<T> SeatInner<T> {
//     fn new(data: T) -> Self {
//         Self { data: Some(data) }
//     }
// }

// impl<T> Default for SeatInner<T> {
//     fn default() -> Self {
//         Self { data: None }
//     }
// }

// #[derive(Debug)]
// pub struct Seat<T> {
//     ptr: NonNull<SeatInner<T>>,
//     rc: Arc<AtomicUsize>,
// }

// impl<T> Seat<T> {
//     pub fn sit(&mut self, data: T) {
//         self.rc.fetch_add(1, Ordering::SeqCst);
//         let ptr = Box::new(SeatInner::new(data));
//         self.ptr = Box::leak(ptr).into();
//     }

//     fn from_inner(ptr: NonNull<SeatInner<T>>) -> Self {
//         Self { ptr, rc: Arc::new(AtomicUsize::new(0)) }
//     }

//     pub fn take(&mut self, j: usize) -> T {
//         self.rc.fetch_sub(1, Ordering::SeqCst);
//         unsafe {
//             let e = mem::replace(&mut self.ptr.as_mut().data, None);
//             match e {
//                 None => panic!("This should have been unreachable {}", j),
//                 Some(e) => e
//             }
//         }
//     }
// }

// impl<T> Drop for Seat<T> {
//     fn drop(&mut self) {
//         unsafe {
//             let cou = self.rc.fetch_sub(1, Ordering::SeqCst);
//             if cou == 0 {
//                 ptr::drop_in_place(self.ptr.as_ptr());
//             }
//         }
//     }
// }

// impl<T> Clone for Seat<T> {
//     fn clone(&self) -> Self {
//         self.rc.fetch_add(1, Ordering::SeqCst);
//         Self::from_inner(self.ptr)
//     }
// }

// impl<T> Default for Seat<T> {
//     fn default() -> Self {
//         let ptr = Box::new(SeatInner::default());
//         Self::from_inner(Box::leak(ptr).into())
//     }
// }

// unsafe impl<T> Send for Seat<T> where T: Send {}
// unsafe impl<T> Sync for Seat<T> where T: Send {}
