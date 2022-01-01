use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::mem;
use std::cell::UnsafeCell;

#[derive(Debug)]
enum Elem<T> {
    Empty,
    Val(T)
}

impl<T> Clone for Elem<T> {
    fn clone(&self) -> Elem<T> {
        match self {
            Elem::Empty => Elem::Empty,
            Elem::Val(_) => unreachable!(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct LFQueue<T> {
    top: Arc<AtomicUsize>,
    bottom: Arc<AtomicUsize>,
    q: Arc<UnsafeCell<Vec<Elem<T>>>>,
}

impl<T> LFQueue<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            top: Arc::new(AtomicUsize::new(0)),
            bottom: Arc::new(AtomicUsize::new(0)),
            q: Arc::new(UnsafeCell::new(vec![Elem::Empty; capacity])),
        }
    }

    fn get_elem(&mut self, idx: usize) -> Option<T> {
        let mut elem = Elem::Empty;
        let q = self.q.get();
        unsafe {
            let len = (*q).len();
            mem::swap(&mut elem, &mut (*q)[idx % len]);    
        }

        match elem {
            Elem::Empty => None,
            Elem::Val(e) => Some(e)
        }
    }

    pub fn push(&mut self, elem: T) {
        let b = self.bottom.load(Ordering::SeqCst);
        let t = self.top.load(Ordering::SeqCst);

        let q = self.q.get();
        unsafe {
            let len = (*q).len();
            if b - t >= len {
                (*q).resize(len * 2, Elem::Empty);
            }

            let len = (*q).len();
            (*q)[b % len] = Elem::Val(elem);    
        }

        self.bottom.store(b + 1, Ordering::SeqCst);
    }

    pub fn pop(&mut self) -> Option<T> {
        let b = self.bottom.fetch_sub(1, Ordering::SeqCst) - 1;
        let t = self.top.load(Ordering::SeqCst);

        if b < t {
            self.bottom.store(t, Ordering::SeqCst);
            return None;
        }
        
        if b > t {
            return self.get_elem(b);
        }

        self.bottom.store(t + 1, Ordering::SeqCst);
        if self.top.compare_exchange(t, t+1, Ordering::SeqCst, Ordering::Relaxed).is_ok() {
            return self.get_elem(b);
        }

        None
    }

    pub fn steal(&mut self) -> Option<T> {
        let b = self.bottom.load(Ordering::SeqCst);
        let t = self.top.load(Ordering::SeqCst);

        if b <= t { return None; }
        if self.top.compare_exchange(t, t+1, Ordering::SeqCst, Ordering::Relaxed).is_ok() {
            return self.get_elem(t);
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

    let s: usize = push_elemts.into_iter().sum();
    let expected_sum = count * s;
    let expected_elems = count * push_elemts.len();
    let mut threads = Vec::new();

    let mut q = LFQueue::new(10);
    let cur = Arc::new(AtomicUsize::new(0));

    for _ in 0..count {
        for e in push_elemts {
            q.push(e);
        }
    }

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

    let mut here = 0;
    loop {
        let e = q.steal();
        match e {
            None => {},
            Some(e) => {
                let _ = cur.fetch_add(e, Ordering::SeqCst); 
                here += 1;
            }
        }

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