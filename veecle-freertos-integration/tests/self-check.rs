#![expect(missing_docs)]

/// Checks that all (non-excluded) tests in this crate follow the special restrictions in README.md.
#[test]
fn verify_tests() -> Result<(), Box<dyn std::error::Error>> {
    let exclusions = ["self-check.rs"];

    let mut error = false;
    for entry in std::fs::read_dir("tests")? {
        let entry = entry?;
        if entry.file_type()?.is_file()
            && !exclusions
                .iter()
                .any(|&exclusion| entry.file_name() == exclusion)
        {
            let content = std::fs::read_to_string(entry.path())?;
            if !content.contains("pub mod common") {
                println!(
                    "{} does not contain `pub mod common`",
                    entry.path().display()
                );
                error = true;
            }
            if content.matches("#[test]").count() > 0 {
                println!(
                    "{} is not using the special test macro",
                    entry.path().display()
                );
                error = true;
            }
        }
    }

    if error {
        println!(
            "some tests aren't following the special FreeRTOS testing requirements, see {}/README.md for details",
            std::env::current_dir()?.display()
        );
        return Err("".into());
    }

    Ok(())
}
