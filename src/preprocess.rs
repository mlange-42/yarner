use crate::util::Fallible;
use std::collections::HashMap;
use std::error::Error;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use yarner_lib::config::Config;
use yarner_lib::document::Document;

pub fn pre_process(
    config: &Config,
    documents: HashMap<PathBuf, Document>,
) -> Fallible<HashMap<PathBuf, Document>> {
    let mut docs = documents;
    for (name, proc) in &config.preprocessor {
        let json = yarner_lib::to_json(proc, &docs)?;

        let command = proc
            .get("command")
            .and_then(|cmd| cmd.as_str().map(|s| s.to_owned()))
            .unwrap_or_else(|| format!("yarner-{}", name));

        let mut child = Command::new(&command)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .map_err(|err| format_error(err.into(), &command))?;

        {
            let stdin = child
                .stdin
                .as_mut()
                .ok_or("Unable to access child process stdin.")
                .map_err(|err| format_error(err.into(), &command))?;
            stdin
                .write_all(json.as_bytes())
                .map_err(|err| format_error(err.into(), &command))?;
        }

        let output = child
            .wait_with_output()
            .map_err(|err| format_error(err.into(), &command))?;
        let out_json =
            String::from_utf8(output.stdout).map_err(|err| format_error(err.into(), &command))?;

        let (_, new_docs) =
            yarner_lib::from_json(&out_json).map_err(|err| format_error(err.into(), &command))?;
        docs = new_docs;
    }
    Ok(docs)
}

fn format_error(err: Box<dyn Error>, name: &str) -> String {
    format!("Failed to run command '{}': {}", name, err.to_string())
}
