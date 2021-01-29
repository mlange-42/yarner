pub mod config;
pub mod document;
pub mod parser;
pub mod templates;
pub mod util;

use crate::config::{AnyConfig, LanguageSettings};
use crate::document::{CompileError, CompileErrorKind, Document};
use crate::parser::{
    code::{CodeParser, RevCodeBlock},
    md::MdParser,
};
use crate::util::Fallible;
use clap::{crate_version, App, Arg, SubCommand};
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

fn main() {
    std::process::exit(match run() {
        Ok(()) => 0,
        Err(err) => {
            eprintln!("{}", err);
            1
        }
    });
}

fn run() -> Fallible {
    let app = App::new("Yarner")
        .version(crate_version!())
        .about("Literate programming compiler\n  \
                  https://github.com/mlange-42/yarner\n\
                \n\
                The normal workflow is:\n \
                1) Create a project with\n    \
                  > yarner create README.md\n \
                2) Process the project by running\n    \
                  > yarner")
        .arg(Arg::with_name("config")
            .short("c")
            .long("config")
            .value_name("config_file")
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
        .arg(Arg::with_name("reverse")
            .long("reverse")
            .short("R")
            .help("Reverse mode: play back code changes into source files.")
            .required(false)
            .takes_value(false))
        .subcommand(SubCommand::with_name("create")
            .about("Creates a yarner project in the current directory")
            .arg(Arg::with_name("file")
                .help("The base file for the doc sources, with normal file extension, but without additional style extension.")
                .value_name("file")
                .takes_value(true)
                .required(true)
                .index(1)));

    let matches = app.clone().get_matches();

    if let Some(matches) = matches.subcommand_matches("create") {
        let file = matches.value_of("file").unwrap();

        match create_project(file) {
            Ok(_) => eprintln!(
                "Successfully created project for {}.\nTo compile the project, run `yarner` from the project directory.",
                file
            ),
            Err(err) => return Err(format!("ERROR: Creating project failed for {}: {}", file, err).into()),
        }

        return Ok(());
    }

    let mut any_config: AnyConfig = match matches.value_of("config") {
        None => AnyConfig::default(),
        Some(file_name) => {
            if matches.occurrences_of("config") == 0 && !PathBuf::from(file_name).exists() {
                AnyConfig::default()
            } else {
                match fs::read_to_string(file_name) {
                    Ok(config) => match toml::from_str(&config) {
                        Ok(config) => config,
                        Err(error) => {
                            return Err(format!(
                                "ERROR: Could not parse config file \"{}\": {}",
                                file_name, error
                            )
                            .into());
                        }
                    },
                    Err(error) => {
                        return Err(format!(
                            "ERROR: Could not read config file \"{}\": {}",
                            file_name, error
                        )
                        .into());
                    }
                }
            }
        }
    };

    any_config.check()?;

    let paths = any_config.paths.clone().unwrap_or_default();

    let root = matches
        .value_of("root")
        .map(|s| s.to_string())
        .or_else(|| paths.root.as_ref().map(|s| s.to_string()))
        .map_or_else(|| PathBuf::from("."), PathBuf::from);

    if let Err(err) = std::env::set_current_dir(&root) {
        return Err(format!(
            "ERROR: --> Unable to set root to \"{}\": {}",
            root.display(),
            err
        )
        .into());
    }

    let reverse = matches.is_present("reverse");
    let clean_code = matches.is_present("clean");
    if let Some(languages) = &mut any_config.language {
        for lang in languages.values_mut() {
            lang.clean_code = clean_code;
        }
    }

    let doc_dir = matches
        .value_of("doc_dir")
        .or_else(|| paths.docs.as_deref())
        .map(PathBuf::from);

    let code_dir = matches
        .value_of("code_dir")
        .or_else(|| paths.code.as_deref())
        .map(PathBuf::from);

    let entrypoint = matches
        .value_of("entrypoint")
        .or_else(|| paths.entrypoint.as_deref());

    let input_patterns: Option<Vec<_>> = matches
        .values_of("input")
        .map(|patterns| patterns.map(|pattern| pattern.to_string()).collect())
        .or_else(|| paths.files.to_owned());

    let input_patterns = input_patterns.ok_or(
        "ERROR: No inputs provided via arguments or toml file. For help, use:\n\
                 > yarner -h",
    )?;

    let language = matches.value_of("language");

    if reverse {
        process_inputs_reverse(
            &input_patterns,
            &any_config,
            &code_dir,
            &doc_dir,
            &entrypoint,
            &language,
        )?;
    } else {
        process_inputs_forward(
            &input_patterns,
            &any_config,
            &code_dir,
            &doc_dir,
            &entrypoint,
            &language,
        )?;
    }

    if let Some(code_dir) = code_dir {
        if let Some(code_file_patterns) = &paths.code_files {
            copy_files(code_file_patterns, &paths.code_paths, code_dir, reverse)?;
        }
    }

    if !reverse {
        if let Some(doc_dir) = doc_dir {
            if let Some(doc_file_patterns) = &paths.doc_files {
                copy_files(doc_file_patterns, &paths.doc_paths, doc_dir, false)?;
            }
        }
    }

    Ok(())
}

fn process_inputs_reverse(
    input_patterns: &[String],
    config: &AnyConfig,
    code_dir: &Option<PathBuf>,
    doc_dir: &Option<PathBuf>,
    entrypoint: &Option<&str>,
    language: &Option<&str>,
) -> Result<(), String> {
    let mut any_input = false;

    let mut documents: HashMap<PathBuf, Document> = HashMap::new();
    let mut code_files: HashSet<PathBuf> = HashSet::new();

    for pattern in input_patterns {
        let paths = match glob::glob(&pattern) {
            Ok(p) => p,
            Err(err) => {
                return Err(format!(
                    "ERROR: --> Unable to process glob pattern \"{}\": {}",
                    pattern, err
                ))
            }
        };
        for path in paths {
            let input = match path {
                Ok(p) => p,
                Err(err) => {
                    return Err(format!(
                        "ERROR: --> Unable to process glob pattern \"{}\": {}",
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

                if let Err(error) = compile_all_reverse(
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
                        "ERROR: Failed to compile source file \"{}\": {}",
                        file_name.display(),
                        error
                    ));
                }
            }
        }
    }

    if !any_input {
        return Err("ERROR: No input files found. For help, use:\n\
                 > yarner -h"
            .to_string());
    }

    reverse(documents, code_files, &config)?;

    Ok(())
}

fn reverse(
    documents: HashMap<PathBuf, Document>,
    code_files: HashSet<PathBuf>,
    config: &AnyConfig,
) -> Result<(), String> {
    let mut code_blocks: HashMap<(PathBuf, Option<String>), Vec<RevCodeBlock>> = HashMap::new();

    let parser = CodeParser {};
    if let Some(languages) = &config.language {
        for file in code_files {
            let language = file.extension().and_then(|s| s.to_str());
            if let Some(language) = language {
                if let Some(lang) = languages.get(language) {
                    let source = fs::read_to_string(&file).map_err(|err| err.to_string())?;
                    let blocks = parser.parse(&source, &config.parser, lang);

                    for block in blocks.into_iter() {
                        let path = PathBuf::from(&block.file);
                        match code_blocks.entry((path, block.name.clone())) {
                            Occupied(mut entry) => {
                                entry.get_mut().push(block);
                            }
                            Vacant(entry) => {
                                entry.insert(vec![block]);
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
            .filter_map(|((p, name), block)| {
                if p == &path {
                    Some((name, block))
                } else {
                    None
                }
            })
            .collect();

        let print = doc.print_reverse(&config.parser, &blocks);

        eprintln!("  --> Writing back to file {}", path.display());

        let mut file = File::create(path).unwrap();
        write!(file, "{}", print).unwrap()
    }

    Ok(())
}

fn process_inputs_forward(
    input_patterns: &[String],
    config: &AnyConfig,
    code_dir: &Option<PathBuf>,
    doc_dir: &Option<PathBuf>,
    entrypoint: &Option<&str>,
    language: &Option<&str>,
) -> Result<(), String> {
    let mut any_input = false;
    for pattern in input_patterns {
        let paths = match glob::glob(&pattern) {
            Ok(p) => p,
            Err(err) => {
                return Err(format!(
                    "ERROR: --> Unable to process glob pattern \"{}\": {}",
                    pattern, err
                ))
            }
        };
        for path in paths {
            let input = match path {
                Ok(p) => p,
                Err(err) => {
                    return Err(format!(
                        "ERROR: --> Unable to process glob pattern \"{}\": {}",
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

                if let Err(error) = compile_all(
                    &parser,
                    doc_dir,
                    code_dir,
                    &file_name,
                    entrypoint,
                    language,
                    &config.language,
                    &mut HashSet::new(),
                    &mut HashSet::new(),
                ) {
                    return Err(format!(
                        "ERROR: Failed to compile source file \"{}\": {}",
                        file_name.display(),
                        error
                    ));
                }
            }
        }
    }

    if !any_input {
        return Err("ERROR: No input files found. For help, use:\n\
                 > yarner -h"
            .to_string());
    }

    Ok(())
}

fn copy_files(
    patterns: &[String],
    path_mod: &Option<Vec<String>>,
    target_dir: PathBuf,
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
                    "ERROR: --> Unable to parse glob pattern \"{}\" (at index {}): {}",
                    file_pattern, err.pos, err
                ))
            }
        };
        for p in paths {
            let file = match p {
                Ok(p) => p,
                Err(err) => {
                    return Err(format!(
                    "ERROR: --> Unable to access result found by glob pattern \"{}\" (at {}): {}",
                    file_pattern,
                    err.path().display(),
                    err
                ))
                }
            };
            if file.is_file() {
                let out_path = path.map_or(file.clone(), |path| modify_path(&file, &path));
                match track_copy_dest.entry(out_path.clone()) {
                    Occupied(entry) => {
                        return Err(format!(
                            "ERROR: Attempted to copy multiple code files to {}: from {} and {}",
                            out_path.display(),
                            entry.get().display(),
                            file.display()
                        ));
                    }
                    Vacant(entry) => {
                        entry.insert(file.clone());
                    }
                }

                let mut file_path = target_dir.clone();
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
                    return Err(format!(
                        "ERROR: --> Error copying file {}: {}",
                        file.display(),
                        err
                    ));
                }
            }
        }
    }
    Ok(())
}

fn modify_path(path: &Path, replace: &str) -> PathBuf {
    if replace.is_empty() || replace == "_" {
        return path.to_owned();
    }
    let mut new_path = PathBuf::new();
    let repl_parts: Vec<_> = Path::new(replace)
        .components()
        .map(|c| c.as_os_str())
        .collect();
    for (i, comp) in path.components().enumerate() {
        if repl_parts.len() > i && repl_parts[i] != "_" {
            if repl_parts[i] != "-" {
                new_path.push(repl_parts[i]);
            }
        } else {
            new_path.push(comp);
        }
    }
    new_path
}

fn create_project(file: &str) -> Fallible {
    let file_name = format!("{}.md", file);
    let base_path = PathBuf::from(&file_name);
    let toml_path = PathBuf::from("Yarner.toml");

    if base_path.exists() {
        return Err(format!("ERROR: File {} already exists.", base_path.display()).into());
    }
    if toml_path.exists() {
        return Err(format!("ERROR: File {} already exists.", toml_path.display()).into());
    }

    let toml = templates::MD_CONFIG.replace("%%MAIN_FILE%%", &file_name);
    let mut toml_file = File::create(&toml_path)?;
    toml_file.write_all(&toml.as_bytes())?;

    let mut base_file = File::create(&base_path)?;
    base_file.write_all(&templates::MD.as_bytes())?;

    Ok(())
}

fn transclude_dry_run(
    parser: &MdParser,
    file_name: &Path,
    code_dir: &Option<PathBuf>,
    entrypoint: &Option<&str>,
    language: &Option<&str>,
    documents: &mut HashMap<PathBuf, Document>,
    track_code_files: &mut HashSet<PathBuf>,
) -> Fallible<Document> {
    let source_main = fs::read_to_string(&file_name)?;
    let document = parser.parse(&source_main, &file_name)?;

    let transclusions = document.tree().transclusions();

    let mut trans_so_far = HashSet::new();
    for trans in transclusions {
        if !trans_so_far.contains(trans.file()) {
            let doc = transclude_dry_run(
                parser,
                trans.file(),
                code_dir,
                entrypoint,
                language,
                documents,
                track_code_files,
            )?;

            compile_reverse(
                parser,
                &doc,
                code_dir,
                trans.file(),
                entrypoint,
                language,
                track_code_files,
            )?;

            documents.insert(trans.file().clone(), doc);
            trans_so_far.insert(trans.file().clone());
        } else {
            return Err(format!("Multiple transclusions of {}", trans.file().display()).into());
        }
    }

    Ok(document)
}

fn transclude(parser: &MdParser, file_name: &Path) -> Fallible<Document> {
    let source_main = fs::read_to_string(&file_name)?;
    let mut document = parser.parse(&source_main, &file_name)?;

    let transclusions = document.tree().transclusions();

    let mut trans_so_far = HashSet::new();
    for trans in transclusions {
        if !trans_so_far.contains(trans.file()) {
            let doc = transclude(parser, trans.file())?;

            // TODO: handle unwrap as error
            let ext = trans.file().extension().unwrap().to_str().unwrap();
            let full_path = trans.file().to_str().unwrap();
            let path = format!(
                "{}{}",
                parser.file_prefix,
                &full_path[..full_path.len() - ext.len() - 1]
            );
            document
                .tree_mut()
                .transclude(&trans, doc.into_tree(), &full_path, &path[..]);

            trans_so_far.insert(trans.file().clone());
        } else {
            return Err(format!("Multiple transclusions of {}", trans.file().display()).into());
        }
    }
    Ok(document)
}

#[allow(clippy::too_many_arguments)]
fn compile_all(
    parser: &MdParser,
    doc_dir: &Option<PathBuf>,
    code_dir: &Option<PathBuf>,
    file_name: &Path,
    entrypoint: &Option<&str>,
    language: &Option<&str>,
    settings: &Option<HashMap<String, LanguageSettings>>,
    track_input_files: &mut HashSet<PathBuf>,
    track_code_files: &mut HashSet<PathBuf>,
) -> Fallible {
    if !track_input_files.contains(file_name) {
        let mut document = transclude(parser, file_name)?;
        let links = parser.find_links(&mut document, file_name, true)?;

        let file_str = file_name.to_str().unwrap();
        document.tree_mut().set_source(file_str);

        compile(
            parser,
            &document,
            doc_dir,
            code_dir,
            file_name,
            entrypoint,
            language,
            settings,
            track_code_files,
        )?;
        track_input_files.insert(file_name.to_owned());

        for file in links {
            if !track_input_files.contains(&file) {
                compile_all(
                    parser,
                    doc_dir,
                    code_dir,
                    &file,
                    entrypoint,
                    language,
                    settings,
                    track_input_files,
                    track_code_files,
                )?;
            }
        }
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn compile_all_reverse(
    parser: &MdParser,
    doc_dir: &Option<PathBuf>,
    code_dir: &Option<PathBuf>,
    file_name: &Path,
    entrypoint: &Option<&str>,
    language: &Option<&str>,
    settings: &Option<HashMap<String, LanguageSettings>>,
    track_input_files: &mut HashSet<PathBuf>,
    track_code_files: &mut HashSet<PathBuf>,
    documents: &mut HashMap<PathBuf, Document>,
) -> Fallible {
    if !track_input_files.contains(file_name) {
        let mut document = transclude_dry_run(
            parser,
            file_name,
            code_dir,
            entrypoint,
            language,
            documents,
            track_code_files,
        )?;
        let links = parser.find_links(&mut document, file_name, false)?;

        let file_str = file_name.to_str().unwrap();
        document.tree_mut().set_source(file_str);

        compile_reverse(
            parser,
            &document,
            code_dir,
            file_name,
            entrypoint,
            language,
            track_code_files,
        )?;

        documents.insert(file_name.to_owned(), document);

        track_input_files.insert(file_name.to_owned());

        for file in links {
            if !track_input_files.contains(&file) {
                compile_all_reverse(
                    parser,
                    doc_dir,
                    code_dir,
                    &file,
                    entrypoint,
                    language,
                    settings,
                    track_input_files,
                    track_code_files,
                    documents,
                )?;
            }
        }
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn compile(
    parser: &MdParser,
    document: &Document,
    doc_dir: &Option<PathBuf>,
    code_dir: &Option<PathBuf>,
    file_name: &Path,
    entrypoint: &Option<&str>,
    language: &Option<&str>,
    settings: &Option<HashMap<String, LanguageSettings>>,
    track_code_files: &mut HashSet<PathBuf>,
) -> Fallible {
    eprintln!("Compiling file {}", file_name.display());

    let mut entries = vec![(entrypoint.as_deref(), file_name.to_owned())];
    let extra_entries = parser.get_entry_points(&document, language);

    entries.extend(
        extra_entries
            .iter()
            .map(|(e, p)| (Some(&e[..]), PathBuf::from((*p).to_owned() + ".temp"))),
    );

    match doc_dir {
        Some(doc_dir) => {
            let documentation = document.print_docs(parser);
            let mut file_path = doc_dir.clone();
            file_path.push(file_name);
            fs::create_dir_all(file_path.parent().unwrap()).unwrap();
            let mut doc_file = File::create(file_path).unwrap();
            write!(doc_file, "{}", documentation).unwrap();
        }
        None => eprintln!("WARNING: Missing output location for docs, skipping docs output."),
    }

    for (entrypoint, sub_file_name) in entries {
        match code_dir {
            Some(code_dir) => {
                let mut file_path = code_dir.clone();
                if let Some(par) = sub_file_name.parent() {
                    file_path.push(par)
                }
                file_path.push(sub_file_name.file_stem().unwrap());
                if let Some(language) = language {
                    file_path.set_extension(language);
                }

                let extension = file_path
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .unwrap_or("")
                    .to_string();
                let settings = match settings {
                    Some(set) => set.get(&extension),
                    None => None,
                };

                if track_code_files.contains(&file_path) {
                    return Err(format!(
                        "ERROR: Multiple locations point to code file {}",
                        file_path.display()
                    )
                    .into());
                } else {
                    track_code_files.insert(file_path.clone());
                }

                match document.print_code(&entrypoint, language, &settings) {
                    Ok(code) => {
                        eprintln!("  --> Writing file {}", file_path.display());
                        fs::create_dir_all(file_path.parent().unwrap()).unwrap();
                        let mut code_file = File::create(file_path).unwrap();
                        write!(code_file, "{}", code).unwrap()
                    }
                    Err(CompileError::Single {
                        kind: CompileErrorKind::MissingEntrypoint,
                        ..
                    }) => {
                        eprintln!(
                            "  --> WARNING: No entrypoint for file {}, skipping code output.",
                            sub_file_name.display()
                        );
                    }
                    Err(err) => return Err(Box::new(err)),
                };
            }
            None => eprintln!("WARNING: Missing output location for code, skipping code output."),
        }
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn compile_reverse(
    parser: &MdParser,
    document: &Document,
    code_dir: &Option<PathBuf>,
    file_name: &Path,
    entrypoint: &Option<&str>,
    language: &Option<&str>,
    track_code_files: &mut HashSet<PathBuf>,
) -> Fallible {
    eprintln!("Compiling file {}", file_name.display());

    let mut entries = vec![(entrypoint.as_deref(), file_name.to_owned())];
    let extra_entries = parser.get_entry_points(&document, language);

    entries.extend(
        extra_entries
            .iter()
            .map(|(e, p)| (Some(&e[..]), PathBuf::from((*p).to_owned() + ".temp"))),
    );

    for (_entrypoint, sub_file_name) in entries {
        match code_dir {
            Some(code_dir) => {
                let mut file_path = code_dir.clone();
                if let Some(par) = sub_file_name.parent() {
                    file_path.push(par)
                }
                file_path.push(sub_file_name.file_stem().unwrap());
                if let Some(language) = language {
                    file_path.set_extension(language);
                }

                if track_code_files.contains(&file_path) {
                    return Err(format!(
                        "ERROR: Multiple locations point to code file {}",
                        file_path.display()
                    )
                    .into());
                } else {
                    track_code_files.insert(file_path.clone());
                }
            }
            None => eprintln!("WARNING: Missing output location for code, skipping code output."),
        }
    }

    Ok(())
}
