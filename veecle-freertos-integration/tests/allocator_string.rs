#![expect(missing_docs)]

pub mod common;

#[common::apply(common::test)]
fn allocator_string() {
    let string = String::from("hello world");

    assert_eq!(string.as_str(), "hello world");
}
