#![expect(missing_docs)]

use freertos_rust::{Duration, FreeRtosError, Queue};

pub mod common;

#[common::apply(common::test)]
fn queue_receive_no_send() {
    let queue: Queue<()> = Queue::new(1).expect("queue to be created");

    common::run_freertos_test(move || {
        assert_eq!(
            queue.receive(Duration::from_ms(0)),
            Err(FreeRtosError::QueueReceiveTimeout)
        );
    })
}
