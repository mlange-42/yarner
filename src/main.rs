mod code;
mod compile;
mod compile_reverse;
mod config;
mod create;
mod document;
mod files;
mod lock;
mod parse;
mod print;
mod util;

use std::collections::{HashMap, HashSet};
use std::env::set_current_dir;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use clap::{crate_version, App, Arg, ArgMatches, SubCommand};

use crate::{
    config::Config,
    document::Document,
    util::{Fallible, JoinExt},
};

fn main() {
    std::process::exit(match run() {
        Ok(()) => 0,
        Err(err) => {
            eprintln!("ERROR: {}", err);
            1
        }
    });
}

fn get_matches<'a>() -> ArgMatches<'a> {
    App::new("Yarner")
        .version(crate_version!())
        .about(r#"Literate Programming tool for Markdown
  https://github.com/mlange-42/yarner

The normal workflow is:
 1) Create a project with
    > yarner init
 2) Process the project by running
    > yarner"#)
        .arg(Arg::with_name("config")
            .short("c")
            .long("config")
            .value_name("path")
            .help("Sets the config file path")
            .takes_value(true)
            .default_value("Yarner.toml"))
        .arg(Arg::with_name("root")
            .long("root")
            .short("r")
            .value_name("path")
            .help("Root directory. Optional. Defaults to 'path -> root' from config file, or to the current directory.")
            .takes_value(true))
        .arg(Arg::with_name("docs")
            .short("d")
            .long("docs")
            .value_name("path")
            .help("Directory to output documentation files to. Optional. Defaults to 'path -> docs' from config file.")
            .takes_value(true))
        .arg(Arg::with_name("code")
            .short("o")
            .long("code")
            .value_name("path")
            .help("Directory to output code files to. Optional. Defaults to 'path -> code' from config file.")
            .takes_value(true))
        .arg(Arg::with_name("entrypoint")
            .short("e")
            .long("entrypoint")
            .value_name("name")
            .help("The named entrypoint to use when tangling code. Optional. Defaults to 'path -> entrypoint', or to the unnamed code block(s).")
            .takes_value(true))
        .arg(Arg::with_name("input")
            .help("The input source file(s) as glob pattern(s). Optional. Defaults to 'path -> files' from config file.")
            .value_name("FILES")
            .multiple(true)
            .index(1))
        .arg(Arg::with_name("clean")
            .long("clean")
            .short("C")
            .help("Produces clean code output, without block label comments.")
            .required(false)
            .takes_value(false))
        .arg(Arg::with_name("force")
            .long("force")
            .short("F")
            .help("Forces building, although it would result in overwriting changed files.")
            .required(false)
            .takes_value(false))
        .subcommand(SubCommand::with_name("init")
            .about("Creates a yarner project in the current directory")
        )
        .subcommand(SubCommand::with_name("reverse")
            .about("Reverse mode: play back code changes into source files")
        )
        .get_matches()
}

fn run() -> Fallible {
    let matches = get_matches();

    if matches.subcommand_matches("init").is_some() {
        create::create_new_project().map_err(|err| format!("Could not create project: {}", err))?;
        println!("Successfully created project.\nTo compile the project, run 'yarner' from here.",);
        return Ok(());
    }

    let config_path = matches.value_of("config").unwrap();
    let mut config = Config::read(config_path)
        .map_err(|err| format!("Could not read config file \"{}\": {}", config_path, err))?;

    config
        .check()
        .map_err(|err| format!("Invalid config file \"{}\": {}", config_path, err))?;

    let lock_path = PathBuf::from(config_path).with_extension("lock");

    let clean_code = matches.is_present("clean");
    let force = matches.is_present("force");
    for lang in config.language.values_mut() {
        lang.clean_code = clean_code;
    }

    let root = matches
        .value_of("root")
        .or_else(|| config.paths.root.as_deref());

    if let Some(path) = root {
        set_current_dir(path)
            .map_err(|err| format!("Unable to set root to \"{}\": {}", path, err))?;
    }

    if let Some(dir) = matches.value_of("docs") {
        config.paths.docs = Some(PathBuf::from(dir));
    }
    if let Some(dir) = matches.value_of("code") {
        config.paths.code = Some(PathBuf::from(dir));
    }
    if let Some(entry) = matches.value_of("entrypoint") {
        config.paths.entrypoint = Some(entry.to_owned());
    }

    let input_patterns = matches
        .values_of("input")
        .map(|patterns| patterns.map(|pattern| pattern.to_owned()).collect())
        .or_else(|| config.paths.files.clone())
        .ok_or(
            "No inputs provided via arguments or toml file. For help, use:\n\
               > yarner -h",
        )?;

    let reverse = matches.subcommand_matches("reverse").is_some();

    if !force
        && config.has_reverse_config()
        && config.paths.has_valid_code_path()
        && lock::files_changed(&lock_path, reverse)?
    {
        return Err(locked_error_message(reverse).into());
    }

    let (mut source_files, mut code_files) = if reverse {
        process_inputs_reverse(&input_patterns, &config)?
    } else {
        process_inputs_forward(&input_patterns, &config)?
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

    if config.has_reverse_config() {
        lock::write_lock(lock_path, &source_files, &code_files)?;
    }

    Ok(())
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

                compile_reverse::compile_all(
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

    let code_blocks = compile_reverse::collect_code_blocks(&code_files, &config)?;
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
            let print = print::print_reverse(&doc, &config.parser, &blocks);
            println!("  Writing back to file {}", path.display());
            let mut file = File::create(path).unwrap();
            write!(file, "{}", print).unwrap()
        } else {
            println!("  Skipping file {}", path.display());
        }
    }

    Ok((source_files, code_files))
}

fn process_inputs_forward(
    input_patterns: &[String],
    config: &Config,
) -> Fallible<(HashSet<PathBuf>, HashSet<PathBuf>)> {
    let mut any_input = false;
    let mut track_source_files = HashSet::new();
    let mut track_code_files = HashMap::new();
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

                compile::compile_all(
                    &config,
                    &file_name,
                    &mut track_source_files,
                    &mut track_code_files,
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

    Ok((
        track_source_files,
        track_code_files.keys().cloned().collect(),
    ))
}
