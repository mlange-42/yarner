use std::collections::HashMap;
use std::error::Error;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use log::{info, warn};
use yarner_lib::{Context, Document, YarnerData, YARNER_VERSION};

use crate::config::Config;
use crate::util::Fallible;

pub fn run_plugins(
    config: &Config,
    documents: HashMap<PathBuf, Document>,
    strict: bool,
) -> Fallible<HashMap<PathBuf, Document>> {
    let mut docs = documents;
    for (name, config) in &config.plugin {
        let command = config
            .get("command")
            .and_then(|cmd| cmd.as_str().map(|s| s.to_owned()))
            .unwrap_or_else(|| format!("yarner-{}", name));

        let arguments: Vec<&str> = config
            .get("arguments")
            .map(|args| {
                args.as_array()
                    .map(|arr| arr.iter().map(|l| l.as_str().unwrap_or_default()).collect())
                    .ok_or("Can't parse array of plugin arguments")
            })
            .transpose()?
            .unwrap_or_default();

        let data = YarnerData {
            context: Context {
                name: name.to_owned(),
                config: config.clone(),
                yarner_version: YARNER_VERSION.to_string(),
            },
            documents: docs,
        };

        let json = to_json(&data)?;

        info!("Running plugin '{}'", name);

        let mut child = Command::new(&command)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .args(&arguments)
            .spawn()
            .map_err(|err| format_error(err.into(), &command))?;

        let has_input = if let Err(err) = child
            .stdin
            .as_mut()
            .ok_or_else(|| "No stdin available.".to_string())
            .and_then(|stdin| {
                stdin
                    .write_all(json.as_bytes())
                    .map_err(|err| err.to_string())
            }) {
            warn!(
                "Plugin '{}' is unable to access child process stdin: {}",
                name,
                err.to_string()
            );

            false
        } else {
            true
        };

        let output = child
            .wait_with_output()
            .map_err(|err| format_error(err.into(), &command))?;

        docs = if output.status.success() {
            if has_input {
                let out_json = String::from_utf8(output.stdout)
                    .map_err(|err| format_error(err.into(), &command))?;

                match from_json(&out_json) {
                    Ok(context) => context.documents,
                    Err(err) => {
                        warn!("Invalid output from plugin '{}': {}", name, err);
                        data.documents
                    }
                }
            } else {
                if !output.stdout.is_empty() {
                    info!("{}", String::from_utf8(output.stdout)?);
                }
                data.documents
            }
        } else {
            if !output.stdout.is_empty() {
                info!("{}", String::from_utf8(output.stdout)?);
            }

            let message = format!(
                "Plugin '{}' exits with error {}.",
                name,
                output.status.code().unwrap_or(1)
            );

            if strict {
                return Err(message.into());
            } else {
                warn!("{}", message);
            }

            data.documents
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
    format!("Failed to run plugin command '{}': {}", name, err)
}
