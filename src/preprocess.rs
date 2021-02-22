use std::collections::HashMap;
use std::error::Error;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use yarner_lib::Document;

use crate::config::Config;
use crate::util::Fallible;
use toml::Value;

pub fn pre_process(
    config: &Config,
    documents: HashMap<PathBuf, Document>,
) -> Fallible<HashMap<PathBuf, Document>> {
    let mut docs = documents;
    for (name, proc) in &config.preprocessor {
        let json = to_json(proc, &docs)?;

        let command = proc
            .get("command")
            .and_then(|cmd| cmd.as_str().map(|s| s.to_owned()))
            .unwrap_or_else(|| format!("yarner-{}", name));

        println!("Running pre-processor {}", command);

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

        docs = from_json(&out_json).map_err(|err| format_error(err.into(), &command))?;
    }
    Ok(docs)
}

fn to_json(config: &Value, documents: &HashMap<PathBuf, Document>) -> serde_json::Result<String> {
    serde_json::to_string_pretty(&(config, documents))
}

fn from_json(json: &str) -> serde_json::Result<HashMap<PathBuf, Document>> {
    serde_json::from_str(json)
}

fn format_error(err: Box<dyn Error>, name: &str) -> String {
    format!("Failed to run command '{}': {}", name, err.to_string())
}
