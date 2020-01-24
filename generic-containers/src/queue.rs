#[macro_export]
macro_rules! impl_queue {
    ($N:literal) => {
        #[derive(Debug)]
        pub enum QueueError {
            Full,
            Empty,
        }

        #[derive(Debug)]
        pub struct Queue<T> {
            start: usize,
            len: usize,
            buf: [Option<T>; $N],
        }

        impl<T> Default for Queue<T> {
            fn default() -> Self {
                Queue::new()
            }
        }

        impl<T> Queue<T> {
            pub fn new() -> Self {
                Queue {
                    start: 0,
                    len: 0,
                    buf: Default::default(),
                }
            }

            pub fn size(&self) -> usize {
                self.len
            }

            pub fn available(&self) -> usize {
                $N - self.len
            }

            pub fn is_empty(&self) -> bool {
                self.len == 0
            }

            pub fn push_back(&mut self, val: T) -> Result<(), QueueError> {
                if self.len >= $N {
                    return Err(QueueError::Full);
                }

                self.buf[(self.start + self.len) % $N] = Some(val);
                self.len += 1;

                Ok(())
            }

            pub fn pop_front(&mut self) -> Result<T, QueueError> {
                if self.len == 0 {
                    return Err(QueueError::Empty);
                }

                let ret = self.buf[self.start].take().unwrap();
                self.start = (self.start + 1) % $N;
                self.len -= 1;

                Ok(ret)
            }

            pub fn peek_front(&self) -> Result<&T, QueueError> {
                if self.len == 0 {
                    return Err(QueueError::Empty);
                }

                Ok(self.buf[self.start].as_ref().unwrap())
            }
        }
    };
}
