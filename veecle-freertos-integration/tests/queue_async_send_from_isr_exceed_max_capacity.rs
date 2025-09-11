#![expect(missing_docs)]

use veecle_freertos_integration::{InterruptContext, channel};

pub mod common;

#[common::apply(common::test)]
fn queue_async_send_from_isr_exceed_max_capacity() {
    let (mut sender, _) = channel::<()>(1).expect("queue to be created");

    common::run_freertos_test(move || {
        let mut interrupt_context = InterruptContext::default();
        assert_eq!(sender.send_from_isr(&mut interrupt_context, ()), Ok(()));
        assert_eq!(sender.send_from_isr(&mut interrupt_context, ()), Err(()));
    });
}
