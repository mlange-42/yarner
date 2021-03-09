use std::{
    collections::{HashMap, HashSet},
    env, fs,
    path::PathBuf,
};

use clap::ArgMatches;
use log::{info, warn};

use yarner_lib::Document;

use crate::{
    code, compile,
    config::Config,
    files, lock, plugin, print,
    util::{Fallible, JoinExt},
};

pub fn run_with_args(
    matches: &ArgMatches,
    reverse_mode: Option<bool>,
    strict: bool,
) -> Fallible<(PathBuf, HashSet<PathBuf>, HashSet<PathBuf>)> {
    let config_path = matches.value_of("config").unwrap();
    let mut config = Config::read(config_path)
        .map_err(|err| format!("Could not read config file \"{}\": {}", config_path, err))?;

    config
        .check()
        .map_err(|err| format!("Invalid config file \"{}\": {}", config_path, err))?;

    let reverse = reverse_mode.unwrap_or_else(|| matches.subcommand_matches("reverse").is_some());
    let has_reverse_config = config.has_reverse_config();

    if reverse && !has_reverse_config {
        let message = "Reverse mode not enabled for any language. Stopping.";
        if strict {
            return Err(message.into());
        } else {
            warn!("{}", message);
        }
    }

    let lock_path = PathBuf::from(config_path).with_extension("lock");

    let clean_code = matches.is_present("clean");
    let force = matches.is_present("force");
    for lang in config.language.values_mut() {
        lang.clean_code = clean_code;
    }

    let root = matches
        .value_of("root")
        .or_else(|| config.paths.root.as_deref());

    let root_path = if let Some(path) = root {
        env::set_current_dir(path)
            .map_err(|err| format!("Unable to set root to \"{}\": {}", path, err))?;
        PathBuf::from(path)
    } else {
        PathBuf::from(".")
    };

    if let Some(dir) = matches.value_of("docs") {
        config.paths.docs = Some(PathBuf::from(dir));
    }
    if let Some(dir) = matches.value_of("code") {
        config.paths.code = Some(PathBuf::from(dir));
    }
    if let Some(entry) = matches.value_of("entrypoint") {
        config.paths.entrypoint = Some(entry.to_owned());
    }
    if let Some(patterns) = matches.values_of("input") {
        config.paths.files = Some(patterns.map(|pattern| pattern.to_owned()).collect());
    }

    let input_patterns = config.paths.files.as_ref().ok_or(
        "No inputs provided via arguments or toml file. For help, use:\n\
               > yarner -h",
    )?;

    if !force
        && has_reverse_config
        && config.paths.has_valid_code_path()
        && lock::files_changed(&lock_path, reverse)?
    {
        return Err(locked_error_message(reverse).into());
    }

    let (mut source_files, mut code_files) = if reverse {
        process_inputs_reverse(&input_patterns, &config)?
    } else {
        process_inputs_forward(&input_patterns, &config, strict)?
    };

    if let (Some(code_dir), Some(code_file_patterns)) =
        (&config.paths.code, &config.paths.code_files)
    {
        let (copy_in, copy_out) = files::copy_files(
            code_file_patterns,
            config.paths.code_paths.as_deref(),
            &code_dir,
            reverse,
        )?;
        source_files.extend(copy_in);
        code_files.extend(copy_out);
    }

    if !reverse {
        if let (Some(doc_dir), Some(doc_file_patterns)) =
            (&config.paths.docs, &config.paths.doc_files)
        {
            files::copy_files(
                doc_file_patterns,
                config.paths.doc_paths.as_deref(),
                &doc_dir,
                false,
            )?;
        }
    }

    if has_reverse_config {
        lock::write_lock(lock_path, &source_files, &code_files)?;
    }

    Ok((
        PathBuf::from(config_path),
        source_files
            .iter()
            .map(|path| root_path.join(path))
            .collect(),
        code_files.iter().map(|path| root_path.join(path)).collect(),
    ))
}

fn locked_error_message(is_reverse: bool) -> String {
    if is_reverse {
        r#"Markdown sources have changed. Stopping to prevent overwrite.
  To run anyway, use `yarner --force reverse`"#
    } else {
        r#"Code output has changed. Stopping to prevent overwrite.
  To run anyway, use `yarner --force`"#
    }
    .to_string()
}

fn process_inputs_reverse(
    input_patterns: &[String],
    config: &Config,
) -> Fallible<(HashSet<PathBuf>, HashSet<PathBuf>)> {
    let code_dir = config.paths.code.as_ref().ok_or({
        r#"Missing code output location. Reverse mode not possible.
  Add 'code = "code"' to section 'path' in file Yarner.toml"#
    })?;

    if !code_dir.exists() {
        return Err(format!(
            r#"Code output target '{}' not found. Reverse mode not possible.
  You may have to run the forward mode first: `yarner`"#,
            code_dir.display()
        )
        .into());
    }
    if !code_dir.is_dir() {
        return Err(format!(
            "Code output target '{}' is not a directory. Reverse mode not possible.",
            code_dir.display()
        )
        .into());
    }

    let mut any_input = false;

    let mut documents: HashMap<PathBuf, Document> = HashMap::new();
    let mut code_files: HashSet<PathBuf> = HashSet::new();
    let mut source_files: HashSet<PathBuf> = HashSet::new();

    for pattern in input_patterns {
        let paths = glob::glob(&pattern)
            .map_err(|err| format!("Unable to process glob pattern \"{}\": {}", pattern, err))?;

        for path in paths {
            let input = path.map_err(|err| {
                format!("Unable to process glob pattern \"{}\": {}", pattern, err)
            })?;

            if input.is_file() {
                any_input = true;
                let file_name = PathBuf::from(&input);

                compile::reverse::compile_all(
                    &config,
                    &file_name,
                    &mut source_files,
                    &mut code_files,
                    &mut documents,
                )
                .map_err(|err| {
                    format!(
                        "Failed to compile source file \"{}\": {}",
                        file_name.display(),
                        err
                    )
                })?
            }
        }
    }

    if !any_input {
        return Err(format!(
            "No input files found in patterns: {}\n\
                For help, use:\n\
                 > yarner -h",
            input_patterns.iter().join(", ", '"')
        )
        .into());
    }

    let code_blocks = code::collect_code_blocks(&code_files, &config)?;
    for (path, doc) in documents {
        let blocks: HashMap<_, _> = code_blocks
            .iter()
            .filter_map(|((p, name, index), block)| {
                if p == &path {
                    Some(((name, index), block))
                } else {
                    None
                }
            })
            .collect();

        if !blocks.is_empty() {
            let print = print::docs::print_reverse(&doc, &config.parser, &blocks);
            if files::file_differs(&path, &print) {
                info!("  Writing back to file {}", path.display());
                fs::write(&path, print)?;
            } else {
                info!("  Skipping unchanged file {}", path.display());
            }
        } else {
            info!("  Skipping file {}", path.display());
        }
    }

    Ok((source_files, code_files))
}

fn process_inputs_forward(
    input_patterns: &[String],
    config: &Config,
    strict: bool,
) -> Fallible<(HashSet<PathBuf>, HashSet<PathBuf>)> {
    let mut any_input = false;
    let mut documents = HashMap::new();
    let mut source_file = HashSet::new();
    for pattern in input_patterns {
        let paths = glob::glob(&pattern)
            .map_err(|err| format!("Unable to process glob pattern \"{}\": {}", pattern, err))?;

        for path in paths {
            let input = path.map_err(|err| {
                format!("Unable to process glob pattern \"{}\": {}", pattern, err)
            })?;

            if input.is_file() {
                any_input = true;
                let file_name = PathBuf::from(&input);

                compile::forward::collect_documents(
                    &config,
                    &file_name,
                    &mut documents,
                    &mut source_file,
                )
                .map_err(|err| {
                    format!(
                        "Failed to compile source file \"{}\": {}",
                        file_name.display(),
                        err
                    )
                })?;
            }
        }
    }

    if !any_input {
        return Err(format!(
            "No input files found in patterns: {}\n\
                For help, use:\n\
                 > yarner -h",
            input_patterns.iter().join(", ", '"')
        )
        .into());
    }

    let code_files = compile::forward::extract_code_all(config, &documents)?;

    let documents = plugin::run_plugins(config, documents, strict)?;
    compile::forward::write_documentation_all(config, &documents)?;

    Ok((source_file, code_files.keys().cloned().collect()))
}
