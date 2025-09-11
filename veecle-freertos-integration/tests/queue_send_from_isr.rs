#![expect(missing_docs)]

use veecle_freertos_integration::{CurrentTask, Duration, InterruptContext, Queue, Task};

pub mod common;

#[common::apply(common::test)]
fn queue_send_from_isr() {
    let queue = Queue::new(1).expect("queue to be created");

    let sender_queue = queue.clone();

    Task::new()
        .start(move |_| {
            let mut interrupt_context = InterruptContext::default();
            sender_queue
                .send_from_isr(&mut interrupt_context, ())
                .expect("message to be sent");

            CurrentTask::delay(Duration::infinite());
        })
        .unwrap();

    common::run_freertos_test(move || {
        assert_eq!(queue.receive(Duration::from_ms(1000)), Ok(()));
    });
}
