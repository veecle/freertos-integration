#![expect(missing_docs)]

use freertos_rust::channel;
use futures::FutureExt;

pub mod common;

#[common::apply(common::test)]
fn queue_async_receive_no_send() {
    let (_, mut receiver) = channel::<()>(1).expect("queue to be created");

    common::run_freertos_test(move || {
        assert_eq!(receiver.receive().now_or_never(), None);
    })
}
