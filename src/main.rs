mod compile;
mod compile_reverse;
mod config;
mod create;
mod document;
mod parse;
mod parser;
mod print;
mod util;

use std::collections::{
    hash_map::Entry::{Occupied, Vacant},
    HashMap, HashSet,
};
use std::env::set_current_dir;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

use clap::{crate_version, App, Arg, SubCommand};

use crate::{
    config::Config,
    document::Document,
    parser::code::{CodeParser, RevCodeBlock},
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

fn run() -> Fallible {
    let app = App::new("Yarner")
        .version(crate_version!())
        .about(r#"Literate programming compiler
  https://github.com/mlange-42/yarner

The normal workflow is:
 1) Create a project with
    > yarner init
 2) Process the project by running
    > yarner"#)
        .arg(Arg::with_name("config")
            .short("c")
            .long("config")
            .value_name("config")
            .help("Sets the config file name")
            .takes_value(true)
            .default_value("Yarner.toml"))
        .arg(Arg::with_name("root")
            .long("root")
            .short("r")
            .value_name("root")
            .help("Root directory. If none is specified, uses 'path' -> 'root' from config file. Default: current directory.")
            .takes_value(true))
        .arg(Arg::with_name("doc_dir")
            .short("d")
            .long("docs")
            .value_name("doc_dir")
            .help("Directory to output weaved documentation files to. If none is specified, uses 'path' -> 'docs' from config file.")
            .takes_value(true))
        .arg(Arg::with_name("code_dir")
            .short("o")
            .long("output")
            .value_name("code_dir")
            .help("Output tangled code files to this directory. If none is specified, uses 'path' -> 'code' from config file.")
            .takes_value(true))
        .arg(Arg::with_name("entrypoint")
            .short("e")
            .long("entrypoint")
            .value_name("entrypoint")
            .help("The named entrypoint to use when tangling code. Defaults to the unnamed code block.")
            .takes_value(true))
        .arg(Arg::with_name("language")
            .short("l")
            .long("language")
            .value_name("language")
            .help("The language to output the tangled code in. Only code blocks in this language will be used.")
            .takes_value(true))
        .arg(Arg::with_name("input")
            .help("The input source file(s) as glob pattern(s). If none are specified, uses 'path' -> 'files' from config file.")
            .value_name("input")
            .multiple(true)
            .index(1))
        .arg(Arg::with_name("clean")
            .long("clean")
            .short("C")
            .help("Produces clean code output, without block label comments.")
            .required(false)
            .takes_value(false))
        .subcommand(SubCommand::with_name("init")
            .about("Creates a yarner project in the current directory")
        )
        .subcommand(SubCommand::with_name("reverse")
            .about("Reverse mode: play back code changes into source files")
        );

    let matches = app.get_matches();

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

    let clean_code = matches.is_present("clean");
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

    let doc_dir = matches
        .value_of("doc_dir")
        .or_else(|| config.paths.docs.as_deref())
        .map(Path::new);

    let code_dir = matches
        .value_of("code_dir")
        .or_else(|| config.paths.code.as_deref())
        .map(Path::new);

    let entrypoint = matches
        .value_of("entrypoint")
        .or_else(|| config.paths.entrypoint.as_deref());

    let input_patterns = matches
        .values_of("input")
        .map(|patterns| patterns.map(|pattern| pattern.to_owned()).collect())
        .or_else(|| config.paths.files.clone())
        .ok_or(
            "No inputs provided via arguments or toml file. For help, use:\n\
               > yarner -h",
        )?;

    let reverse = matches.subcommand_matches("reverse").is_some();

    let language = matches.value_of("language");

    if reverse {
        process_inputs_reverse(
            &input_patterns,
            &config,
            code_dir,
            doc_dir,
            entrypoint,
            language,
        )?;
    } else {
        process_inputs_forward(
            &input_patterns,
            &config,
            code_dir,
            doc_dir,
            entrypoint,
            language,
        )?;
    }

    if let Some(code_dir) = code_dir {
        if let Some(code_file_patterns) = &config.paths.code_files {
            copy_files(
                code_file_patterns,
                config.paths.code_paths.as_deref(),
                code_dir,
                reverse,
            )?;
        }
    }

    if !reverse {
        if let Some(doc_dir) = doc_dir {
            if let Some(doc_file_patterns) = &config.paths.doc_files {
                copy_files(
                    doc_file_patterns,
                    config.paths.doc_paths.as_deref(),
                    doc_dir,
                    false,
                )?;
            }
        }
    }

    Ok(())
}

fn process_inputs_reverse(
    input_patterns: &[String],
    config: &Config,
    code_dir: Option<&Path>,
    doc_dir: Option<&Path>,
    entrypoint: Option<&str>,
    language: Option<&str>,
) -> Fallible {
    let mut any_input = false;

    let mut documents: HashMap<PathBuf, Document> = HashMap::new();
    let mut code_files: HashSet<PathBuf> = HashSet::new();

    for pattern in input_patterns {
        let paths = match glob::glob(&pattern) {
            Ok(p) => p,
            Err(err) => {
                return Err(
                    format!("Unable to process glob pattern \"{}\": {}", pattern, err).into(),
                )
            }
        };
        for path in paths {
            let input = match path {
                Ok(p) => p,
                Err(err) => {
                    return Err(
                        format!("Unable to process glob pattern \"{}\": {}", pattern, err).into(),
                    )
                }
            };
            if input.is_file() {
                any_input = true;
                let (file_name, code_type) = {
                    let file_name = PathBuf::from(&input);

                    let code_type = input.file_stem().and_then(|stem| {
                        PathBuf::from(stem)
                            .extension()
                            .and_then(|osstr| osstr.to_str())
                            .map(|s| s.to_owned())
                    });
                    (file_name, code_type)
                };

                let parser = config.parser.default_language(code_type);

                if let Err(error) = compile_reverse::compile_all(
                    &parser,
                    doc_dir,
                    code_dir,
                    &file_name,
                    entrypoint,
                    language,
                    &config.language,
                    &mut HashSet::new(),
                    &mut code_files,
                    &mut documents,
                ) {
                    return Err(format!(
                        "Failed to compile source file \"{}\": {}",
                        file_name.display(),
                        error
                    )
                    .into());
                }
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

    reverse(documents, code_files, &config)?;

    Ok(())
}

fn reverse(
    documents: HashMap<PathBuf, Document>,
    code_files: HashSet<PathBuf>,
    config: &Config,
) -> Fallible {
    let mut code_blocks: HashMap<(PathBuf, Option<String>, usize), RevCodeBlock> = HashMap::new();

    let parser = CodeParser {};
    if !config.language.is_empty() {
        for file in code_files {
            let language = file.extension().and_then(|s| s.to_str());
            if let Some(language) = language {
                if let Some(labels) = config
                    .language
                    .get(language)
                    .and_then(|lang| lang.block_labels.as_ref())
                {
                    let source = util::read_file(&file)?;
                    let blocks = parser.parse(&source, &config.parser, labels)?;

                    for block in blocks.into_iter() {
                        let path = PathBuf::from(&block.file);
                        match code_blocks.entry((path, block.name.clone(), block.index)) {
                            Occupied(entry) => {
                                if entry.get().lines != block.lines {
                                    return Err(format!("Reverse mode impossible due to multiple, differing occurrences of a code block: {} # {} # {}",
                                                       &block.file, &block.name.unwrap_or_else(|| "".to_string()), block.index).into());
                                } else {
                                    eprintln!("  WARNING: multiple occurrences of a code block: {} # {} # {}", &block.file, &block.name.unwrap_or_else(|| "".to_string()), block.index)
                                }
                            }
                            Vacant(entry) => {
                                entry.insert(block);
                            }
                        }
                    }
                }
            }
        }
    }

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
            eprintln!("  Writing back to file {}", path.display());
            let mut file = File::create(path).unwrap();
            write!(file, "{}", print).unwrap()
        } else {
            eprintln!("  Skipping file {}", path.display());
        }
    }

    Ok(())
}

fn process_inputs_forward(
    input_patterns: &[String],
    config: &Config,
    code_dir: Option<&Path>,
    doc_dir: Option<&Path>,
    entrypoint: Option<&str>,
    language: Option<&str>,
) -> Result<(), String> {
    let mut any_input = false;
    let mut track_code_files = HashMap::new();
    for pattern in input_patterns {
        let paths = match glob::glob(&pattern) {
            Ok(p) => p,
            Err(err) => {
                return Err(format!(
                    "Unable to process glob pattern \"{}\": {}",
                    pattern, err
                ))
            }
        };
        for path in paths {
            let input = match path {
                Ok(p) => p,
                Err(err) => {
                    return Err(format!(
                        "Unable to process glob pattern \"{}\": {}",
                        pattern, err
                    ))
                }
            };
            if input.is_file() {
                any_input = true;
                let (file_name, code_type) = {
                    let file_name = PathBuf::from(&input);

                    let code_type = input.file_stem().and_then(|stem| {
                        PathBuf::from(stem)
                            .extension()
                            .and_then(|osstr| osstr.to_str())
                            .map(|s| s.to_owned())
                    });
                    (file_name, code_type)
                };

                let parser = config.parser.default_language(code_type);

                if let Err(error) = compile::compile_all(
                    &parser,
                    doc_dir,
                    code_dir,
                    &file_name,
                    entrypoint,
                    language,
                    &config.language,
                    &mut HashSet::new(),
                    &mut track_code_files,
                ) {
                    return Err(format!(
                        "Failed to compile source file \"{}\": {}",
                        file_name.display(),
                        error
                    ));
                }
            }
        }
    }

    if !any_input {
        return Err(format!(
            "No input files found in patterns: {}\n\
                For help, use:\n\
                 > yarner -h",
            input_patterns.iter().join(", ", '"')
        ));
    }

    Ok(())
}

fn copy_files(
    patterns: &[String],
    path_mod: Option<&[String]>,
    target_dir: &Path,
    reverse: bool,
) -> Result<(), String> {
    match path_mod {
        Some(path_mod) if patterns.len() != path_mod.len() => {
            return Err(
                "If argument code_paths/doc_paths is given in the toml file, it must have as many elements as argument code_files/doc_files".to_string()
            );
        }
        _ => (),
    }
    let mut track_copy_dest: HashMap<PathBuf, PathBuf> = HashMap::new();
    for (idx, file_pattern) in patterns.iter().enumerate() {
        let path = path_mod.as_ref().map(|paths| &paths[idx]);
        let paths = match glob::glob(&file_pattern) {
            Ok(p) => p,
            Err(err) => {
                return Err(format!(
                    "Unable to parse glob pattern \"{}\" (at index {}): {}",
                    file_pattern, err.pos, err
                ))
            }
        };
        for p in paths {
            let file = match p {
                Ok(p) => p,
                Err(err) => {
                    return Err(format!(
                        "Unable to access result found by glob pattern \"{}\" (at {}): {}",
                        file_pattern,
                        err.path().display(),
                        err
                    ))
                }
            };
            if file.is_file() {
                let out_path = path.map_or(file.clone(), |path| util::modify_path(&file, &path));
                match track_copy_dest.entry(out_path.clone()) {
                    Occupied(entry) => {
                        return Err(format!(
                            "Attempted to copy multiple code files to {}: from {} and {}",
                            out_path.display(),
                            entry.get().display(),
                            file.display()
                        ));
                    }
                    Vacant(entry) => {
                        entry.insert(file.clone());
                    }
                }

                let mut file_path = target_dir.to_owned();
                file_path.push(out_path);

                if !reverse {
                    fs::create_dir_all(file_path.parent().unwrap()).unwrap();
                }
                let (from, to) = if reverse {
                    eprintln!("Copying file {} to {}", file_path.display(), file.display());
                    (&file_path, &file)
                } else {
                    eprintln!("Copying file {} to {}", file.display(), file_path.display());
                    (&file, &file_path)
                };
                if let Err(err) = fs::copy(&from, &to) {
                    return Err(format!("Error copying file {}: {}", file.display(), err));
                }
            }
        }
    }
    Ok(())
}
