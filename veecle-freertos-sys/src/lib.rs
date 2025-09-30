//! FreeRTOS low-level Rust bindings.
#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(test)]
extern crate std;

pub mod error;
mod macro_wrappers;
mod safe_wrappers;

#[expect(non_upper_case_globals)]
#[expect(non_camel_case_types)]
#[expect(non_snake_case)]
#[allow(missing_docs)]
#[expect(missing_debug_implementations)]
#[cfg_attr(docsrs, doc = include_str!(concat!(env!("OUT_DIR"), "/warning.md")))]
pub mod bindings {
    pub use crate::macro_wrappers::*;
    pub use crate::safe_wrappers::*;
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

#[cfg(test)]
mod tests {
    use similar_asserts::SimpleDiff;
    use std::path::PathBuf;

    #[test]
    #[ignore] // This is system dependent so only run in CI.
    fn check_sample_posix_bindings() -> Result<(), ()> {
        let sample_path = PathBuf::from(env!("CARGO_MANIFEST_PATH"))
            .parent()
            .unwrap()
            .join("src/posix-sample-bindings.rs");
        let actual_path = PathBuf::from(env!("OUT_DIR")).join("bindings.rs");

        let sample = std::fs::read_to_string(&sample_path).unwrap();
        let actual = std::fs::read_to_string(&actual_path).unwrap();

        if sample != actual {
            if std::env::var("BLESS").as_deref() == Ok("1") {
                std::fs::copy(actual_path, sample_path).unwrap();
            } else {
                std::println!(
                    "sample bindings have changed, rerun tests with BLESS=1 to update (see README.md for more details): \n\n{}",
                    SimpleDiff::from_str(&sample, &actual, "sample", "actual",),
                );
                return Err(());
            }
        }

        Ok(())
    }
}
