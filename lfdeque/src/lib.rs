use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::cell::UnsafeCell;
mod seat;
use seat::Seat;

#[derive(Debug, Clone)]
pub struct LFQueue<T> {
    top: Arc<AtomicUsize>,
    bottom: Arc<AtomicUsize>,
    q: Arc<UnsafeCell<Vec<Option<Seat<T>>>>>,
}

impl<T: std::fmt::Debug> LFQueue<T> {
    pub fn new(capacity: usize) -> Self {
        let q = Arc::new(UnsafeCell::new(Vec::new()));
        let temp = q.get();
        unsafe {
            (*temp).resize_with(capacity, || None);
        }

        Self {
            top: Arc::new(AtomicUsize::new(0)),
            bottom: Arc::new(AtomicUsize::new(0)),
            q
        }
    }

    fn get_elem(&mut self, idx: usize) -> Option<Seat<T>> {
        let q = self.q.get();
        unsafe {
            let len = (*q).len();
            let elem = &(*q)[idx % len];
            elem.as_ref().cloned()
        }
    }

    pub fn push(&mut self, elem: T) -> bool {
        let b = self.bottom.load(Ordering::SeqCst);
        let t = self.top.load(Ordering::SeqCst);

        let q = self.q.get();
        unsafe {
            let len = (*q).len();
            if b - t >= len - 1 {
                return false
            }

            (*q)[b % len] = Some(Seat::new(elem));
        }

        self.bottom.store(b + 1, Ordering::SeqCst);
        true
    }

    pub fn pop(&mut self) -> Option<T> {
        let b = self.bottom.fetch_sub(1, Ordering::SeqCst) - 1;
        let t = self.top.load(Ordering::SeqCst);

        if b < t {
            self.bottom.store(t, Ordering::SeqCst);
            return None;
        }
        
        let elem = self.get_elem(b);
        if b > t {
            return elem.map(Seat::take)
        }

        self.bottom.store(t + 1, Ordering::SeqCst);
        if self.top.compare_exchange(t, t+1, Ordering::SeqCst, Ordering::SeqCst).is_ok() {
            return elem.map(Seat::take);
        }

        None
    }

    pub fn steal(&mut self) -> Option<T> {
        let b = self.bottom.load(Ordering::SeqCst);
        let t = self.top.load(Ordering::SeqCst);

        if b <= t { return None; }
        let elem = self.get_elem(t);
        if self.top.compare_exchange(t, t+1, Ordering::SeqCst, Ordering::SeqCst).is_ok() {
            return elem.map(Seat::take)
        }

        None
    }
}

unsafe impl<T> Send for LFQueue<T> where T: Send {}
unsafe impl<T> Sync for LFQueue<T> where T: Send {}

#[test]
fn test_init() {
    use std::thread;

    let push_elemts = [1, 2, 93, 104, 2044];
    let count =  100000;
    let thread_count = 7;

    let s: usize = push_elemts.iter().sum();
    let expected_sum = count * s;
    let expected_elems = count * push_elemts.len();
    let mut threads = Vec::new();

    let mut q = LFQueue::new(5);
    let cur = Arc::new(AtomicUsize::new(0));

    for i in 0..thread_count {
        let mut q_clone = q.clone();
        let cur_clone = cur.clone();
    
        let t = thread::spawn(move || {
            let mut here = 0;
            loop {
                let e = q_clone.steal();
                match e {
                    None => {},
                    Some(e) => { 
                        let _ = cur_clone.fetch_add(e, Ordering::SeqCst);
                        here += 1;
                    }
                }
        
                let c = cur_clone.load(Ordering::SeqCst);
                if c == expected_sum { break; }
            }

            (i, here)
        });

        threads.push(t);
    }

    for _ in 0..count {
        for e in push_elemts {
            let mut r = q.push(e);
            while !r {
                r = q.push(e);
            }
        }
    }

    let mut here = 0;
    loop {
        let e = q.pop();
        match e {
            None => {},
            Some(e) => {
                let _ = cur.fetch_add(e, Ordering::SeqCst); 
                here += 1;
            }
        }
        println!("main {:?}", e);
        let c = cur.load(Ordering::SeqCst);
        if c == expected_sum { break; }
    }

    let mut exec_count = 0;
    println!("thrd\texecs");
    for t in threads {
        let (t_id, count) = t.join().unwrap();
        println!("{}\t{}", t_id, count);
        exec_count += count;
    }
    println!("main\t{}", here);
    exec_count += here;
    println!("total\t{}", exec_count);

    assert_eq!(cur.load(Ordering::SeqCst), expected_sum);
    assert_eq!(exec_count, expected_elems);
    assert_eq!(q.pop(), None);
    assert_eq!(q.steal(), None);
}