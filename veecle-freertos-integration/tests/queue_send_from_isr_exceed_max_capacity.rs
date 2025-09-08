#![expect(missing_docs)]

use freertos_rust::{InterruptContext, Queue};

pub mod common;

#[common::apply(common::test)]
fn queue_send_from_isr_exceed_max_capacity() {
    let queue = Queue::new(1).expect("queue to be created");

    common::run_freertos_test(move || {
        let mut interrupt_context = InterruptContext::default();
        queue
            .send_from_isr(&mut interrupt_context, ())
            .expect("message to be sent");
        assert_eq!(queue.send_from_isr(&mut interrupt_context, ()), Err(()));
    });
}
