use crate::util::Fallible;
use std::fs::OpenOptions;
use std::io::Write;

pub fn create_new_project(main_file: &str) -> Fallible {
    let main_file = format!("{}.md", main_file);

    let mut document = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&main_file)?;

    let mut config = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open("Yarner.toml")?;

    document.write_all(DOCUMENT.as_bytes())?;

    config.write_all(CONFIG.replace("%%MAIN_FILE%%", &main_file).as_bytes())?;

    Ok(())
}

const DOCUMENT: &str = include_str!("document.md");

const CONFIG: &str = include_str!("Yarner.toml");
