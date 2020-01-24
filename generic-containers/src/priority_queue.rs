//! TODO: reimplement as a binary heap

#[macro_export]
macro_rules! impl_priority_queue {
    ($MAX_PRIORITY:literal, $N:literal) => {
        #[allow(dead_code)]
        mod q {
            $crate::impl_queue!($N);
        }
        use q::Queue;

        #[derive(Debug)]
        pub enum PriorityQueueError {
            Full,
            BadPriority,
        }

        pub struct PriorityQueue<T> {
            queues: [Queue<T>; $MAX_PRIORITY],
        }

        impl<T> Default for PriorityQueue<T> {
            fn default() -> Self {
                PriorityQueue::new()
            }
        }

        impl<T> PriorityQueue<T> {
            pub fn new() -> Self {
                PriorityQueue {
                    queues: Default::default(),
                }
            }

            pub fn is_empty(&self) -> bool {
                self.queues.iter().all(|q| q.is_empty())
            }

            pub fn push(&mut self, val: T, priority: usize) -> Result<(), (T, PriorityQueueError)> {
                if priority > $MAX_PRIORITY {
                    return Err((val, PriorityQueueError::BadPriority));
                }

                if let Err(val) = self.queues[priority].push_back(val) {
                    return Err((val, PriorityQueueError::Full));
                }

                Ok(())
            }

            pub fn pop(&mut self) -> Option<T> {
                for q in self.queues.iter_mut().rev() {
                    if let Some(val) = q.pop_front() {
                        return Some(val);
                    }
                }
                None
            }
        }
    };
}
