use clap::{crate_authors, crate_version, App, Arg, SubCommand};
use either::Either::{self, *};
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use yarner::config::{AnyConfig, LanguageSettings};
use yarner::document::{CompileError, CompileErrorKind, Document};
use yarner::parser::{HtmlParser, MdParser, Parser, ParserConfig, Printer, TexParser};
use yarner::{templates, MultipleTransclusionError, ProjectCreationError};

fn main() {
    let matches = App::new("Yarner")
        .version(crate_version!())
        .author(crate_authors!())
        .about("Literate programming compiler")
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
            .help("The input source file(s). If none are specified, uses 'path' -> 'files' from config file.")
            .value_name("input")
            .multiple(true)
            .index(1))
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
                .index(2)))
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("create") {
        let file = matches.value_of("file").unwrap();
        let style = matches.value_of("style").unwrap();

        match create_project(file, style) {
            Ok(_) => eprintln!(
                "Successfully created project for {}.\nTo compile the project, run `yarner` from the project directory.",
                file
            ),
            Err(err) => eprintln!("Creating project failed for {}: {}", file, err),
        }

        return;
    }

    let any_config: AnyConfig = match matches.value_of("config") {
        None => AnyConfig::default(),
        Some(file_name) => {
            if matches.occurrences_of("config") == 0 && !PathBuf::from(file_name).exists() {
                AnyConfig::default()
            } else {
                match fs::read_to_string(file_name) {
                    Ok(config) => match toml::from_str(&config) {
                        Ok(config) => config,
                        Err(error) => {
                            eprintln!("Could not parse config file \"{}\": {}", file_name, error);
                            return;
                        }
                    },
                    Err(error) => {
                        eprintln!("Could not read config file \"{}\": {}", file_name, error);
                        return;
                    }
                }
            }
        }
    };

    let paths = any_config.paths.unwrap_or_default();

    let doc_dir = match matches.subcommand_matches("weave") {
        Some(..) => Left(()),
        None => Right(
            matches
                .value_of("doc_dir")
                .map(|s| s.to_string())
                .or_else(|| paths.docs.as_ref().map(|s| s.to_string()))
                .map(PathBuf::from),
        ),
    };
    let code_dir = match matches.subcommand_matches("tangle") {
        Some(..) => Left(()),
        None => Right(
            matches
                .value_of("code_dir")
                .map(|s| s.to_string())
                .or_else(|| paths.code.as_ref().map(|s| s.to_string()))
                .map(PathBuf::from),
        ),
    };
    let entrypoint = matches
        .value_of("entrypoint")
        .map(|ep| ep)
        .or_else(|| paths.entrypoint.as_deref());

    enum Input {
        File(String),
        //Stdin,
    }

    let inputs = matches.values_of("input").map(|files| {
        files
            .into_iter()
            .map(|file| Input::File(file.to_string()))
            .collect()
    });

    let inputs: Vec<_> = match inputs.or_else(|| {
        paths
            .files
            .as_ref()
            .map(|files| files.iter().map(|file| Input::File(file.clone())).collect())
    }) {
        Some(inputs) => inputs,
        None => {
            eprintln!("No inputs provided via arguments or toml file.",);
            return;
        }
    };

    for input in inputs {
        let (file_name, style_type, code_type) = match input {
            Input::File(file_name) => {
                let file_name = PathBuf::from(file_name);

                let style_type = file_name
                    .extension()
                    .and_then(|osstr| osstr.to_str())
                    .map(|s| s.to_owned());

                let code_type = file_name.file_stem().and_then(|stem| {
                    PathBuf::from(stem)
                        .extension()
                        .and_then(|osstr| osstr.to_str())
                        .map(|s| s.to_owned())
                });
                (file_name, style_type, code_type)
            }
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
                ) {
                    eprintln!(
                        "Failed to compile source file \"{}\": {}",
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
                ) {
                    eprintln!(
                        "Failed to compile source file \"{}\": {}",
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
                ) {
                    eprintln!(
                        "Failed to compile source file \"{}\": {}",
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
}

fn create_project(file: &str, style: &str) -> Result<(), Box<dyn Error>> {
    let file_name = format!("{}.{}", file, style);
    let base_path = PathBuf::from(&file_name);
    let toml_path = PathBuf::from("Yarner.toml");

    if base_path.exists() {
        return Err(Box::new(ProjectCreationError(format!(
            "File {:?} already exists.",
            base_path
        ))));
    }
    if toml_path.exists() {
        return Err(Box::new(ProjectCreationError(format!(
            "File {:?} already exists.",
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
    doc_dir: &Either<(), Option<PathBuf>>,
    code_dir: &Either<(), Option<PathBuf>>,
    file_name: &PathBuf,
    entrypoint: Option<&str>,
    language: Option<&str>,
    settings: &Option<HashMap<String, LanguageSettings>>,
    all_files: &mut HashSet<PathBuf>,
) -> Result<(), Box<dyn std::error::Error>>
where
    P: Parser + Printer,
    P::Error: 'static,
{
    if !all_files.contains(file_name) {
        let mut document = transclude(parser, file_name, None)?;
        let links = parser.find_links(&document, file_name)?;

        let file_str = file_name.to_str().unwrap();
        document.tree_mut().set_source(file_str);

        compile(
            parser, &document, doc_dir, code_dir, &file_name, entrypoint, language, settings,
        )?;
        all_files.insert(file_name.clone());

        for file in links {
            if !all_files.contains(&file) {
                compile_all(
                    parser, doc_dir, code_dir, &file, entrypoint, language, settings, all_files,
                )?;
            }
        }
    }

    Ok(())
}

fn compile<P>(
    parser: &P,
    document: &Document,
    doc_dir: &Either<(), Option<PathBuf>>,
    code_dir: &Either<(), Option<PathBuf>>,
    file_name: &PathBuf,
    entrypoint: Option<&str>,
    language: Option<&str>,
    settings: &Option<HashMap<String, LanguageSettings>>,
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
        Left(..) => {
            let documentation = document.print_docs(parser);
            print!("{}", documentation);
        }
        Right(Some(doc_dir)) => {
            let documentation = document.print_docs(parser);
            let mut file_path = doc_dir.clone();
            file_path.push(file_name);
            fs::create_dir_all(file_path.parent().unwrap()).unwrap();
            let mut doc_file = File::create(file_path).unwrap();
            write!(doc_file, "{}", documentation).unwrap();
        }
        _ => {}
    }

    for (entrypoint, sub_file_name) in entries {
        match code_dir {
            Left(..) => {
                let extension = sub_file_name
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .unwrap_or("")
                    .to_string();
                let settings = match settings {
                    Some(set) => set.get(&extension),
                    None => None,
                };

                let code = document.print_code(entrypoint, language, settings)?;
                println!("{}", code);
            }
            Right(Some(code_dir)) => {
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
            _ => {}
        }
    }

    Ok(())
}
