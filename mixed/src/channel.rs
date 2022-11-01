use std::{
    collections::VecDeque,
    sync::{atomic::AtomicUsize, Arc, Condvar, Mutex},
};

use anyhow::{anyhow, Result};

struct Shared<T> {
    queue: Mutex<VecDeque<T>>,
    available: Condvar,
    senders: AtomicUsize,
    receivers: AtomicUsize,
}
pub struct Sender<T> {
    shared: Arc<Shared<T>>,
}

pub struct Receiver<T> {
    shared: Arc<Shared<T>>,
    cache: VecDeque<T>,
}

impl<T> Sender<T> {
    pub fn send(&mut self, t: T) -> Result<()> {
        if self.total_receivers() == 0 {
            return Err(anyhow!("no receiver left"));
        }

        let was_empty = {
            let mut inner = self.shared.queue.lock().unwrap();
            let empty = inner.is_empty();
            inner.push_back(t);
            empty
        };
        if was_empty {
            self.shared.available.notify_one();
        }
        Ok(())
    }

    pub fn total_receivers(&self) -> usize {
        self.shared
            .receivers
            .load(std::sync::atomic::Ordering::SeqCst)
    }

    pub fn total_queued_items(&self) -> usize {
        let queue = self.shared.queue.lock().unwrap();
        queue.len()
    }
}

impl<T> Receiver<T> {
    pub fn recv(&mut self) -> Result<T> {
        if let Some(v) = self.cache.pop_front() {
            return Ok(v);
        }
        let mut inner = self.shared.queue.lock().unwrap();
        loop {
            match inner.pop_front() {
                Some(t) => {
                    if !inner.is_empty() {
                        std::mem::swap(&mut self.cache, &mut inner);
                    }
                    return Ok(t);
                }
                None if self.total_senders() == 0 => return Err(anyhow!("no sender left")),
                None => {
                    inner = self
                        .shared
                        .available
                        .wait(inner)
                        .map_err(|_| anyhow!("lock poisoned"))?;
                }
            }
        }
    }

    pub fn total_senders(&self) -> usize {
        self.shared
            .senders
            .load(std::sync::atomic::Ordering::SeqCst)
    }
}

impl<T> Iterator for Receiver<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        self.recv().ok()
    }
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        self.shared
            .senders
            .fetch_add(1, std::sync::atomic::Ordering::AcqRel);
        Self {
            shared: Arc::clone(&self.shared),
        }
    }
}

impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        let old = self
            .shared
            .senders
            .fetch_sub(1, std::sync::atomic::Ordering::AcqRel);
        if old <= 1 {
            self.shared.available.notify_all();
        }
    }
}

impl<T> Drop for Receiver<T> {
    fn drop(&mut self) {
        self.shared
            .receivers
            .fetch_sub(1, std::sync::atomic::Ordering::AcqRel);
    }
}

pub fn unbounded<T>() -> (Sender<T>, Receiver<T>) {
    let shared = Shared::default();
    let shared = Arc::new(shared);
    (
        Sender {
            shared: shared.clone(),
        },
        Receiver {
            shared,
            cache: VecDeque::with_capacity(10),
        },
    )
}

impl<T> Default for Shared<T> {
    fn default() -> Self {
        Self {
            queue: Mutex::new(VecDeque::with_capacity(10)),
            available: Condvar::new(),
            senders: AtomicUsize::new(1),
            receivers: AtomicUsize::new(1),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::thread;
    use std::time::Duration;

    use super::*;

    #[test]
    fn channel_should_work() {
        let (mut s, mut r) = unbounded();
        s.send("hello world!".to_string()).unwrap();
        let msg = r.recv().unwrap();
        assert_eq!(msg, "hello world!");
    }

    #[test]
    fn multiple_senders_should_work() {
        let (mut s, mut r) = unbounded();
        let mut s1 = s.clone();
        let mut s2 = s.clone();
        let t = thread::spawn(move || {
            s.send(1).unwrap();
        });
        let t1 = thread::spawn(move || {
            s1.send(2).unwrap();
        });
        let t2 = thread::spawn(move || {
            s2.send(3).unwrap();
        });

        for handle in [t, t1, t2] {
            handle.join().unwrap();
        }

        let mut result = [r.recv().unwrap(), r.recv().unwrap(), r.recv().unwrap()];
        result.sort();
        assert_eq!(result, [1, 2, 3]);
    }
    #[test]
    fn receiver_should_be_blocked_when_nothing_to_read() {
        let (mut s, r) = unbounded();
        let mut s1 = s.clone();
        thread::spawn(move || {
            for (idx, i) in r.into_iter().enumerate() {
                // 如果读到数据，确保它和发送的数据一致
                assert_eq!(idx, i)
            }
            // 读不到应该休眠，所以不会执行到这一句，执行到这一句说明逻辑出错
            assert!(false)
        });
        thread::spawn(move || {
            for i in 0..100usize {
                s.send(i).unwrap();
            }
        });
        // 1ms 足够让生产者发完 100 个消息，消费者消费完 100 个消息并阻塞
        thread::sleep(Duration::from_millis(1));
        for i in 100..200usize {
            s1.send(i).unwrap();
        }
        thread::sleep(Duration::from_millis(1));
        assert_eq!(s1.total_queued_items(), 0);
    }
    #[test]
    fn last_sender_drop_should_error_when_receive() {
        let (s, mut r) = unbounded();
        let s1 = s.clone();
        let senders = [s, s1];
        let total = senders.len();

        // sender 即用即抛
        for mut sender in senders {
            thread::spawn(move || {
                sender.send("hello").unwrap();
            })
            .join()
            .unwrap();
        }
        // 虽然没有 sender 了，接收者依然可以接受已经在队列里的数据
        for _ in 0..total {
            r.recv().unwrap();
        }
        // 然而，读取更多数据时会出错
        assert!(r.recv().is_err());
    }
    #[test]
    fn receiver_drop_should_error_when_send() {
        let (mut s1, mut s2) = {
            let (s, _) = unbounded();
            let s1 = s.clone();
            let s2 = s.clone();
            (s1, s2)
        };
        assert!(s1.send(1).is_err());
        assert!(s2.send(1).is_err());
    }
    #[test]
    fn receiver_shall_be_notified_when_all_senders_exit() {
        let (s, mut r) = unbounded::<usize>();
        let (mut sender, mut receiver) = unbounded::<usize>();
        let t1 = thread::spawn(move || {
            sender.send(0).unwrap();
            assert!(r.recv().is_err());
        });

        thread::spawn(move || {
            receiver.recv().unwrap();
            drop(s);
        });
        t1.join().unwrap();
    }
    #[test]
    fn channel_fast_path_should_work() {
        let (mut s, mut r) = unbounded();
        for i in 0..10usize {
            s.send(i).unwrap();
        }
        assert!(r.cache.is_empty());

        assert_eq!(0, r.recv().unwrap());
        assert_eq!(r.cache.len(), 9);
        assert_eq!(s.total_queued_items(), 0);

        for (idx, i) in r.into_iter().take(9).enumerate() {
            assert_eq!(idx + 1, i);
        }
    }
}
