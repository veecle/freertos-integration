#![expect(missing_docs)]

use freertos_rust::Queue;

pub mod common;

#[common::apply(common::test)]
fn queue_raw() {
    let queue: Queue<()> = Queue::new(1).expect("queue to be created");

    let raw_queue_handle = queue.raw_handle();

    // SAFETY: The handle is a valid handle for a `Queue<()>`.
    let from_raw_queue: Queue<()> = unsafe { Queue::from_raw_handle(raw_queue_handle) };
    assert_eq!(queue.raw_handle(), from_raw_queue.raw_handle());
}
