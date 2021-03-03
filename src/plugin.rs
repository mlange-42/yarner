use std::collections::HashMap;
use std::error::Error;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use yarner_lib::{Context, Document, YarnerData, YARNER_VERSION};

use crate::config::Config;
use crate::util::Fallible;

pub fn pre_process(
    config: &Config,
    documents: HashMap<PathBuf, Document>,
) -> Fallible<HashMap<PathBuf, Document>> {
    let mut docs = documents;
    for (name, config) in &config.plugin {
        let command = config
            .get("command")
            .and_then(|cmd| cmd.as_str().map(|s| s.to_owned()))
            .unwrap_or_else(|| format!("yarner-{}", name));

        let arguments: Vec<&str> = match config.get("arguments") {
            None => vec![],
            Some(v) => v
                .as_array()
                .map(|arr| arr.iter().map(|l| l.as_str().unwrap_or_default()))
                .ok_or("Can't parse array of plugin arguments")?
                .collect(),
        };

        let command_string = format!(
            "{}{}{}",
            command,
            if arguments.is_empty() { "" } else { " " },
            &arguments.join(" "),
        );

        let data = YarnerData {
            context: Context {
                name: name.to_owned(),
                config: config.clone(),
                yarner_version: YARNER_VERSION.to_string(),
            },
            documents: docs,
        };

        let json = to_json(&data)?;

        println!("Running plugin command '{}'", command_string);

        let mut child = Command::new(&command)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .args(&arguments)
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

        if !output.status.success() {
            return Err(format!("Plugin command '{}' exits with error.", command_string,).into());
        }

        let out_json =
            String::from_utf8(output.stdout).map_err(|err| format_error(err.into(), &command))?;

        docs = match from_json(&out_json) {
            Ok(context) => context.documents,
            Err(err) => {
                eprintln!(
                    "Warning: Invalid output from plugin command '{}': {}",
                    command_string, err
                );
                data.documents
            }
        }
    }
    Ok(docs)
}

fn to_json(data: &YarnerData) -> serde_json::Result<String> {
    serde_json::to_string_pretty(&data)
}

fn from_json(json: &str) -> serde_json::Result<YarnerData> {
    serde_json::from_str(json)
}

fn format_error(err: Box<dyn Error>, name: &str) -> String {
    format!(
        "Failed to run plugin command '{}': {}",
        name,
        err.to_string()
    )
}
