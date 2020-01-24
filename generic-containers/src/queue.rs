#[macro_export]
macro_rules! impl_queue {
    ($N:literal) => {
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

            /// Returns back `val` if the vector is full.
            pub fn push_back(&mut self, val: T) -> Result<(), T> {
                if self.len >= $N {
                    return Err(val);
                }

                self.buf[(self.start + self.len) % $N] = Some(val);
                self.len += 1;

                Ok(())
            }

            pub fn pop_front(&mut self) -> Option<T> {
                if self.len == 0 {
                    return None;
                }

                let ret = self.buf[self.start].take().unwrap();
                self.start = (self.start + 1) % $N;
                self.len -= 1;

                Some(ret)
            }

            pub fn peek_front(&self) -> Option<&T> {
                if self.len == 0 {
                    return None;
                }

                Some(self.buf[self.start].as_ref().unwrap())
            }
        }
    };
}
