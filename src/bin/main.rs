use clap::{crate_version, App, Arg, SubCommand};
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use yarner::config::{AnyConfig, LanguageSettings};
use yarner::document::{CompileError, CompileErrorKind, Document};
use yarner::parser::{HtmlParser, MdParser, ParseError, Parser, ParserConfig, Printer, TexParser};
use yarner::util::PathUtil;
use yarner::{templates, MultipleTransclusionError, ProjectCreationError};

fn main() {
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
        .arg(Arg::with_name("style")
            .short("s")
            .long("style")
            .value_name("style")
            .help("Sets the style to use. If not specified, it is inferred from the file extension.")
            .takes_value(true)
            .possible_values(&["md", "tex", "html"]))
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
        .subcommand(SubCommand::with_name("create")
            .about("Creates a yarner project in the current directory")
            .arg(Arg::with_name("file")
                .help("The base file for the doc sources, with normal file extension, but without additional style extension.")
                .value_name("file")
                .takes_value(true)
                .required(true)
                .index(1))
            .arg(Arg::with_name("style")
                .short("s")
                .help("Sets the style to use.")
                .takes_value(true)
                .possible_values(&["md", "tex", "html"])
                .default_value("md")
                .index(2)));

    let matches = app.clone().get_matches();

    if let Some(matches) = matches.subcommand_matches("create") {
        let file = matches.value_of("file").unwrap();
        let style = matches.value_of("style").unwrap();

        match create_project(file, style) {
            Ok(_) => eprintln!(
                "Successfully created project for {}.\nTo compile the project, run `yarner` from the project directory.",
                file
            ),
            Err(err) => eprintln!("ERROR: Creating project failed for {}: {}", file, err),
        }

        return;
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
                            eprintln!(
                                "ERROR: Could not parse config file \"{}\": {}",
                                file_name, error
                            );
                            return;
                        }
                    },
                    Err(error) => {
                        eprintln!(
                            "ERROR: Could not read config file \"{}\": {}",
                            file_name, error
                        );
                        return;
                    }
                }
            }
        }
    };

    let paths = any_config.paths.unwrap_or_default();

    let clean_code = matches.is_present("clean");
    if let Some(languages) = &mut any_config.language {
        for lang in languages.values_mut() {
            lang.clean_code = clean_code;
        }
    }

    let doc_dir = matches
        .value_of("doc_dir")
        .map(|s| s.to_string())
        .or_else(|| paths.docs.as_ref().map(|s| s.to_string()))
        .map(PathBuf::from);

    let code_dir = matches
        .value_of("code_dir")
        .map(|s| s.to_string())
        .or_else(|| paths.code.as_ref().map(|s| s.to_string()))
        .map(PathBuf::from);

    let entrypoint = matches
        .value_of("entrypoint")
        .map(|ep| ep)
        .or_else(|| paths.entrypoint.as_deref());

    let inputs = matches.values_of("input").map(|files| {
        PathUtil::list_all_files_str(&files.into_iter().collect::<Vec<_>>()[..])
            .unwrap()
            .into_iter()
            .map(|file| file.clone())
            .collect()
    });

    let inputs: Vec<_> = match inputs.or_else(|| {
        paths.files.as_ref().map(|files| {
            PathUtil::list_all_files(&files[..])
                .unwrap()
                .into_iter()
                .map(|file| file.clone())
                .collect()
        })
    }) {
        Some(inputs) => inputs,
        None => {
            eprintln!(
                "No inputs provided via arguments or toml file. For help, use:\n\
                 > yarner -h",
            );
            return;
        }
    };

    for input in inputs {
        let (file_name, style_type, code_type) = {
            let file_name = PathBuf::from(&input);

            let style_type = input
                .extension()
                .and_then(|osstr| osstr.to_str())
                .map(|s| s.to_owned());

            let code_type = input.file_stem().and_then(|stem| {
                PathBuf::from(stem)
                    .extension()
                    .and_then(|osstr| osstr.to_str())
                    .map(|s| s.to_owned())
            });
            (file_name, style_type, code_type)
        };

        let language = matches.value_of("language");

        match matches
            .value_of("style")
            .map(|s| s.to_string())
            .or(style_type)
            .unwrap_or("md".to_string())
            .as_str()
        {
            "md" => {
                let default = MdParser::default();
                let parser = any_config
                    .md
                    .as_ref()
                    .unwrap_or(&default)
                    .default_language(code_type);
                if let Err(error) = compile_all(
                    &parser,
                    &doc_dir,
                    &code_dir,
                    &file_name,
                    entrypoint,
                    language,
                    &any_config.language,
                    &mut HashSet::new(),
                    &mut HashSet::new(),
                ) {
                    eprintln!(
                        "ERROR: Failed to compile source file \"{}\": {}",
                        file_name.to_str().unwrap(),
                        error
                    );
                    continue;
                }
            }
            "tex" => {
                let default = TexParser::default();
                let parser = any_config
                    .tex
                    .as_ref()
                    .unwrap_or(&default)
                    .default_language(code_type);
                if let Err(error) = compile_all(
                    &parser,
                    &doc_dir,
                    &code_dir,
                    &file_name,
                    entrypoint,
                    language,
                    &any_config.language,
                    &mut HashSet::new(),
                    &mut HashSet::new(),
                ) {
                    eprintln!(
                        "ERROR: Failed to compile source file \"{}\": {}",
                        file_name.to_str().unwrap(),
                        error
                    );
                    continue;
                }
            }
            "html" => {
                let default = HtmlParser::default();
                let parser = any_config
                    .html
                    .as_ref()
                    .unwrap_or(&default)
                    .default_language(code_type);
                if let Err(error) = compile_all(
                    &parser,
                    &doc_dir,
                    &code_dir,
                    &file_name,
                    entrypoint,
                    language,
                    &any_config.language,
                    &mut HashSet::new(),
                    &mut HashSet::new(),
                ) {
                    eprintln!(
                        "ERROR: Failed to compile source file \"{}\": {}",
                        file_name.to_str().unwrap(),
                        error
                    );
                    continue;
                }
            }
            other => {
                eprintln!("Unknown style {}", other);
                continue;
            }
        };
    }

    match (&paths.code_files, &paths.code_paths) {
        (Some(code_files), Some(code_paths)) if code_files.len() != code_paths.len() => {
            eprintln!(
                "If argument code_paths is given in the toml file, it must have as many elements as argument code_files",
            );
            return;
        }
        _ => (),
    }

    let mut track_copy_dest = HashMap::new();
    if let Some(code_dir) = code_dir {
        if let Some(code_file_patterns) = paths.code_files {
            for (idx, code_file_pattern) in code_file_patterns.iter().enumerate() {
                let code_path = paths.code_paths.as_ref().map(|code_paths| &code_paths[idx]);
                for code_file in PathUtil::list_files(code_file_pattern).unwrap() {
                    let out_path = code_path.map_or(code_file.clone(), |code_path| {
                        modify_path(&code_file, &code_path)
                    });
                    match track_copy_dest.entry(out_path.clone()) {
                        Occupied(entry) => {
                            eprintln!(
                                "ERROR: Attempted to copy multiple code files to {:?}: from {:?} and {:?}",
                                out_path, entry.get(), code_file
                            );
                            return;
                        }
                        Vacant(entry) => {
                            entry.insert(code_file.clone());
                        }
                    }
                    let mut file_path = code_dir.clone();
                    file_path.push(out_path.clone());

                    eprintln!("Copying code file {:?} to {:?}", code_file, out_path);
                    fs::create_dir_all(file_path.parent().unwrap()).unwrap();
                    match fs::copy(&code_file, &file_path) {
                        Ok(_) => {}
                        Err(err) => eprintln!(
                            "ERROR: --> Error copying code file {:?}: {}",
                            code_file, err
                        ),
                    }
                }
            }
        }
    }

    if let Some(doc_dir) = doc_dir {
        if let Some(doc_files) = paths.doc_files {
            for doc_file in PathUtil::list_all_files(&doc_files[..]).unwrap() {
                eprintln!("Copying doc file {:?}", doc_file);
                let mut file_path = doc_dir.clone();
                file_path.push(&doc_file);
                fs::create_dir_all(file_path.parent().unwrap()).unwrap();
                match fs::copy(&doc_file, &file_path) {
                    Ok(_) => {}
                    Err(err) => eprintln!(
                        "ERROR: --> Problem copying doc file {:?}: {}",
                        doc_file, err
                    ),
                }
            }
        }
    }
}

fn modify_path(path: &PathBuf, replace: &str) -> PathBuf {
    if replace.is_empty() || replace == "_" {
        return path.clone();
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

fn create_project(file: &str, style: &str) -> Result<(), Box<dyn Error>> {
    let file_name = format!("{}.{}", file, style);
    let base_path = PathBuf::from(&file_name);
    let toml_path = PathBuf::from("Yarner.toml");

    if base_path.exists() {
        return Err(Box::new(ProjectCreationError(format!(
            "ERROR: File {:?} already exists.",
            base_path
        ))));
    }
    if toml_path.exists() {
        return Err(Box::new(ProjectCreationError(format!(
            "ERROR: File {:?} already exists.",
            toml_path
        ))));
    }

    let (template, toml) = match style {
        "md" => (templates::MD, templates::MD_CONFIG),
        "tex" => (templates::TEX, templates::TEX_CONFIG),
        "html" => (templates::HTML, templates::HTML_CONFIG),
        _ => ("", ""),
    };

    let toml = toml.replace("%%MAIN_FILE%%", &file_name);
    let mut toml_file = File::create(&toml_path)?;
    toml_file.write_all(&toml.as_bytes())?;

    let mut base_file = File::create(&base_path)?;
    base_file.write_all(&template.as_bytes())?;

    Ok(())
}

fn transclude<P>(
    parser: &P,
    file_name: &PathBuf,
    into: Option<&PathBuf>,
) -> Result<Document, Box<dyn std::error::Error>>
where
    P: Parser + Printer + ParserConfig,
    P::Error: 'static,
{
    let file = match into {
        Some(into) => {
            let mut path = into.parent().unwrap().to_path_buf();
            path.push(file_name);
            path
        }
        None => file_name.to_owned(),
    };
    let source_main = fs::read_to_string(&file)?;
    let mut document = parser.parse(&source_main)?;

    let transclusions = document.tree().transclusions();

    let mut trans_so_far = HashSet::new();
    for trans in transclusions {
        if !trans_so_far.contains(trans.file()) {
            let doc = transclude(parser, trans.file(), Some(&file))?;

            // TODO: handle unwrap as error
            let ext = trans.file().extension().unwrap().to_str().unwrap();
            let full_path = trans.file().to_str().unwrap();
            let path = format!(
                "{}{}",
                parser.file_prefix(),
                &full_path[..full_path.len() - ext.len() - 1]
            );
            document
                .tree_mut()
                .transclude(&trans, doc.into_tree(), &full_path, &path[..]);

            trans_so_far.insert(trans.file().clone());
        } else {
            return Err(Box::new(MultipleTransclusionError(trans.file().clone())));
        }
    }
    Ok(document)
}

fn compile_all<P>(
    parser: &P,
    doc_dir: &Option<PathBuf>,
    code_dir: &Option<PathBuf>,
    file_name: &PathBuf,
    entrypoint: Option<&str>,
    language: Option<&str>,
    settings: &Option<HashMap<String, LanguageSettings>>,
    track_input_files: &mut HashSet<PathBuf>,
    track_code_files: &mut HashSet<PathBuf>,
) -> Result<(), Box<dyn std::error::Error>>
where
    P: Parser + Printer,
    P::Error: 'static,
{
    if !track_input_files.contains(file_name) {
        let mut document = transclude(parser, file_name, None)?;
        let links = parser.find_links(&document, file_name)?;

        let file_str = file_name.to_str().unwrap();
        document.tree_mut().set_source(file_str);

        compile(
            parser,
            &document,
            doc_dir,
            code_dir,
            &file_name,
            entrypoint,
            language,
            settings,
            track_code_files,
        )?;
        track_input_files.insert(file_name.clone());

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

fn compile<P>(
    parser: &P,
    document: &Document,
    doc_dir: &Option<PathBuf>,
    code_dir: &Option<PathBuf>,
    file_name: &PathBuf,
    entrypoint: Option<&str>,
    language: Option<&str>,
    settings: &Option<HashMap<String, LanguageSettings>>,
    track_code_files: &mut HashSet<PathBuf>,
) -> Result<(), Box<dyn std::error::Error>>
where
    P: Parser + Printer,
    P::Error: 'static,
{
    eprintln!("Compiling file {:?}", file_name);

    let mut entries = vec![(entrypoint, file_name.clone())];
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
                    return Err(Box::new(ParseError::MultipleCodeFileAccessError(format!(
                        "ERROR: Multiple locations point to code file {:?}",
                        file_path
                    ))));
                } else {
                    track_code_files.insert(file_path.clone());
                }

                match document.print_code(entrypoint, language, settings) {
                    Ok(code) => {
                        eprintln!("  --> Writing file {:?}", file_path);
                        fs::create_dir_all(file_path.parent().unwrap()).unwrap();
                        let mut code_file = File::create(file_path).unwrap();
                        write!(code_file, "{}", code).unwrap()
                    }
                    Err(err) => match &err {
                        CompileError::Single {
                            line_number: _,
                            kind,
                        } => match kind {
                            CompileErrorKind::MissingEntrypoint => {
                                eprintln!(
                                    "  --> WARNING: No entrypoint for file {:?}, skipping code output.",
                                    sub_file_name
                                );
                            }
                            _ => return Err(Box::new(err)),
                        },
                        _ => return Err(Box::new(err)),
                    },
                };
            }
            None => eprintln!("WARNING: Missing output location for code, skipping code output."),
        }
    }

    Ok(())
}
