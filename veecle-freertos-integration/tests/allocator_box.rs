#![expect(missing_docs)]

pub mod common;

#[common::apply(common::test)]
fn allocator_box() {
    #[repr(align(4096))]
    struct Aligned(usize);

    let aligned = Box::new(Aligned(1));

    assert!((&raw const aligned).is_aligned());
    assert_eq!(aligned.0, 1);
}
