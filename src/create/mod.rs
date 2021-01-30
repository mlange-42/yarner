use crate::util::Fallible;
use std::fs::{remove_file, OpenOptions};
use std::io::Write;
use std::mem::forget;

pub fn create_new_project() -> Fallible {
    let mut config = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open("Yarner.toml")?;

    let remove_config = RemoveOnDrop("Yarner.toml");

    let mut document = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open("README.md")?;

    let remove_document = RemoveOnDrop("README.md");

    config.write_all(CONFIG.as_bytes())?;

    document.write_all(DOCUMENT.as_bytes())?;

    forget(remove_config);
    forget(remove_document);

    Ok(())
}

struct RemoveOnDrop<'a>(&'a str);

impl Drop for RemoveOnDrop<'_> {
    fn drop(&mut self) {
        let _ = remove_file(self.0);
    }
}

const CONFIG: &str = include_str!("Yarner.toml");

const DOCUMENT: &str = include_str!("README.md");
