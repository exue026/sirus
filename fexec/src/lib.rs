use lfdeque::LFQueue;

pub struct SwapQueue<T> {
    waitq: LFQueue<T>,
    activeq: LFQueue<T>,
}

impl<T> SwapQueue<T> {
    pub fn new() -> Self {
        let mut waitq = LFQueue::new(10);
        let mut activeq = LFQueue::new(10);

        Self {
            waitq,
            activeq
        }
    }
}