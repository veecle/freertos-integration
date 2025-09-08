#![expect(missing_docs)]

use freertos_rust::channel;
use futures::FutureExt;

pub mod common;

#[common::apply(common::test)]
fn queue_async_send_exceed_max_capacity() {
    let (mut sender, _) = channel::<()>(1).expect("queue to be created");

    common::run_freertos_test(move || {
        assert_eq!(sender.send(()).now_or_never(), Some(()));
        assert_eq!(sender.send(()).now_or_never(), None);
    })
}
