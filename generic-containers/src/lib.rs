#![no_std]

#[macro_use]
mod queue;

#[macro_use]
mod priority_queue;

/// Example generated structures.
pub mod examples {
    mod q_16 {
        impl_queue!(16);
    }

    pub use {q_16::Queue as Queue_16, q_16::QueueError as Queue_16Error};

    mod pq_8_16 {
        impl_priority_queue!(8, 16);
    }

    pub use {
        pq_8_16::PriorityQueue as PriorityQueue_8_16,
        pq_8_16::PriorityQueueError as PriorityQueue_8_16Error,
    };
}
