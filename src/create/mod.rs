use crate::util::Fallible;
use std::fs::{remove_file, OpenOptions};
use std::io::Write;
use std::mem::forget;

pub fn create_new_project(main_file: &str) -> Fallible {
    let main_file = format!("{}.md", main_file);

    let mut config = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open("Yarner.toml")?;

    let remove_config = RemoveOnDrop("Yarner.toml");

    let mut document = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&main_file)?;

    let remove_document = RemoveOnDrop(&main_file);

    config.write_all(CONFIG.replace("%%MAIN_FILE%%", &main_file).as_bytes())?;

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

const DOCUMENT: &str = include_str!("document.md");
