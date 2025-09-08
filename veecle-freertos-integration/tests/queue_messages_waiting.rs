#![expect(missing_docs)]

use freertos_rust::{CurrentTask, Duration, Queue, Task};

pub mod common;

#[common::apply(common::test)]
fn queue_messages_waiting() {
    let queue = Queue::new(1).expect("queue to be created");

    let sender_queue = queue.clone();

    Task::new()
        .start(move |_| {
            assert_eq!(sender_queue.messages_waiting(), 0);
            sender_queue
                .send((), Duration::from_ms(1000))
                .expect("message to be sent");
            assert_eq!(sender_queue.messages_waiting(), 1);

            CurrentTask::delay(Duration::infinite());
        })
        .unwrap();

    common::run_freertos_test(move || {
        assert_eq!(queue.receive(Duration::from_ms(1000)), Ok(()));
        assert_eq!(queue.messages_waiting(), 0);
    });
}
