#![expect(missing_docs)]

pub mod common;

use veecle_freertos_integration::Duration;

#[common::apply(common::test)]
fn units() {
    assert_eq!(Duration::zero().ms(), 0);
    assert_eq!(Duration::zero().ticks(), 0);

    assert!(Duration::eps().ms() > 0);
    assert!(Duration::eps().ticks() > 0);

    assert!(Duration::max().ms() > 0);
    assert!(Duration::max().ticks() > 0);

    assert!(Duration::infinite().ms() > 0);
    assert!(Duration::infinite().ticks() > 0);

    assert_eq!(Duration::from_ms(0), Duration::zero());
    assert_eq!(Duration::from_ms(100), Duration::from_ms(100));
    assert_eq!(Duration::from_ms(100).ms(), 100);

    assert_eq!(Duration::from_ticks(0), Duration::zero());
    assert_eq!(Duration::from_ticks(100), Duration::from_ticks(100));
    assert_eq!(Duration::from_ticks(100).ticks(), 100);
}
