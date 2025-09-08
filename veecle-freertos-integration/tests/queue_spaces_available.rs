#![expect(missing_docs)]

use freertos_rust::{CurrentTask, Duration, Queue, Task};

pub mod common;

#[common::apply(common::test)]
fn queue_spaces_available() {
    let queue = Queue::new(1).expect("queue to be created");

    let sender_queue = queue.clone();

    Task::new()
        .start(move |_| {
            assert_eq!(sender_queue.spaces_available(), 1);
            sender_queue
                .send((), Duration::from_ms(1000))
                .expect("message to be sent");
            assert_eq!(sender_queue.spaces_available(), 0);

            CurrentTask::delay(Duration::infinite());
        })
        .unwrap();

    common::run_freertos_test(move || {
        assert_eq!(queue.receive(Duration::from_ms(1000)), Ok(()));
        assert_eq!(queue.spaces_available(), 1);
    });
}
