use clap::{App, Arg, SubCommand};
use either::Either::{self, *};
use outline::config::{AnyConfig, Paths};
use outline::parser::{BirdParser, HtmlParser, MdParser, Parser, Printer, TexParser};
use outline::ProjectCreationError;
use std::error::Error;
use std::fs::{self, File};
use std::io::{stdin, Read, Write};
use std::path::PathBuf;

fn main() {
    let matches = App::new("Outline")
        .version("1.0")
        .author("Cameron Eldridge <cameldridge@gmail.com>")
        .about("Literate programming compiler")
        .arg(Arg::with_name("config")
            .short("c")
            .long("config")
            .value_name("config_file")
            .help("Sets the config file name")
            .takes_value(true)
            .default_value("Outline.toml"))
        .arg(Arg::with_name("style")
            .short("s")
            .long("style")
            .value_name("style")
            .help("Sets the style to use. If not specified, it is inferred from the file extension. When reading from STDIN, defaults to 'md'.")
            .takes_value(true)
            .possible_values(&["bird", "md", "tex", "html"]))
        .arg(Arg::with_name("doc_dir")
            .short("d")
            .long("docs")
            .value_name("doc_dir")
            .help("Directory to output weaved documentation files to. No documentation will be printed by default.")
            .takes_value(true))
        .arg(Arg::with_name("code_dir")
            .short("o")
            .long("output")
            .value_name("code_dir")
            .help("Output tangled code files to this directory. No code files will be printed by default.")
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
            .help("The input source file(s). If none are specified, read from STDIN, and print generated code to STDOUT.")
            .value_name("input")
            .multiple(true)
            .index(1))
        .subcommand(SubCommand::with_name("tangle")
            .about("Tangle input and print to STDOUT")
            .arg(Arg::with_name("input")
                .help("The input source file(s). If none are specified, read from STDIN")
                .value_name("input")
                .multiple(true)
                .index(1)))
        .subcommand(SubCommand::with_name("weave")
            .about("Weave input and print to STDOUT")
            .arg(Arg::with_name("input")
                .help("The input source file(s). If none are specified, read from STDIN")
                .value_name("input")
                .multiple(true)
                .index(1)))
        .subcommand(SubCommand::with_name("create")
            .about("Creates an outline project in the current directory")
            .arg(Arg::with_name("file")
                .help("The base for the doc sources, with normal file extension, but without additional style extension.")
                .value_name("file")
                .takes_value(true)
                .required(true)
                .index(1))
            .arg(Arg::with_name("style")
                .short("s")
                .help("Sets the style to use.")
                .takes_value(true)
                .possible_values(&["bird", "md", "tex", "html"])
                .default_value("md")
                .index(2)))
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("create") {
        let file = matches.value_of("file").unwrap();
        let style = matches.value_of("style").unwrap();

        match create_project(file, style) {
            Ok(_) => eprintln!(
                "Successfully created project for {}.\nTo compile the project, run `outline` from the project directory.",
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

    enum Input {
        File(String),
        Stdin,
    }

    let inputs = matches
        .subcommand_matches("weave")
        .or(matches.subcommand_matches("tangle"))
        .unwrap_or(&matches)
        .values_of("input")
        .map(|files| {
            files
                .into_iter()
                .map(|file| Input::File(file.to_string()))
                .collect()
        });

    let inputs = inputs
        .or_else(|| {
            paths
                .files
                .as_ref()
                .map(|files| files.iter().map(|file| Input::File(file.clone())).collect())
        })
        .unwrap_or_else(|| vec![Input::Stdin]);

    for input in inputs {
        let (file_name, contents, style_type, code_type) = match input {
            Input::File(file_name) => {
                let file_name = PathBuf::from(file_name);

                let contents = match fs::read_to_string(&file_name) {
                    Ok(contents) => contents,
                    Err(error) => {
                        eprintln!(
                            "Could not read source file \"{}\": {}",
                            file_name.to_str().unwrap(),
                            error
                        );
                        return;
                    }
                };

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
                (Some(file_name), contents, style_type, code_type)
            }
            Input::Stdin => {
                let mut input = String::new();
                match stdin().read_to_string(&mut input) {
                    Ok(..) => (),
                    Err(error) => {
                        eprintln!("Could not read STDIN as string: {}", error);
                        return;
                    }
                }
                (None, input, None, None)
            }
        };

        let language = matches.value_of("language");
        let entrypoint = matches.value_of("entrypoint");

        match matches
            .value_of("style")
            .map(|s| s.to_string())
            .or(style_type)
            .unwrap_or("md".to_string())
            .as_str()
        {
            "bird" => {
                let default = BirdParser::default();
                let parser = any_config.bird.as_ref().unwrap_or(&default);
                if let Err(error) = compile(
                    parser, &contents, &doc_dir, &code_dir, &file_name, entrypoint, language,
                ) {
                    if let Some(file_name) = file_name {
                        eprintln!(
                            "Failed to compile source file \"{}\": {}",
                            file_name.to_str().unwrap(),
                            error
                        );
                    } else {
                        eprintln!("Failed to compile from STDIN: {}", error);
                        std::process::exit(1);
                    }
                    continue;
                }
            }
            "md" => {
                let default = MdParser::default();
                let parser = any_config
                    .md
                    .as_ref()
                    .unwrap_or(&default)
                    .default_language(code_type);
                if let Err(error) = compile(
                    &parser, &contents, &doc_dir, &code_dir, &file_name, entrypoint, language,
                ) {
                    if let Some(file_name) = file_name {
                        eprintln!(
                            "Failed to compile source file \"{}\": {}",
                            file_name.to_str().unwrap(),
                            error
                        );
                    } else {
                        eprintln!("Failed to compile from STDIN: {}", error);
                        std::process::exit(1);
                    }
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
                if let Err(error) = compile(
                    &parser, &contents, &doc_dir, &code_dir, &file_name, entrypoint, language,
                ) {
                    if let Some(file_name) = file_name {
                        eprintln!(
                            "Failed to compile source file \"{}\": {}",
                            file_name.to_str().unwrap(),
                            error
                        );
                    } else {
                        eprintln!("Failed to compile from STDIN: {}", error);
                        std::process::exit(1);
                    }
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
                if let Err(error) = compile(
                    &parser, &contents, &doc_dir, &code_dir, &file_name, entrypoint, language,
                ) {
                    if let Some(file_name) = file_name {
                        eprintln!(
                            "Failed to compile source file \"{}\": {}",
                            file_name.to_str().unwrap(),
                            error
                        );
                    } else {
                        eprintln!("Failed to compile from STDIN: {}", error);
                        std::process::exit(1);
                    }
                    continue;
                }
            }
            other => {
                eprintln!("Unknown style {}", other);
                if file_name.is_none() {
                    std::process::exit(1);
                }
                continue;
            }
        };
    }
}

fn create_project(file: &str, style: &str) -> Result<(), Box<dyn Error>> {
    let file_name = format!("{}.{}", file, style);
    let base_path = PathBuf::from(&file_name);
    let toml_path = PathBuf::from("Outline.toml");

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

    let mut config = AnyConfig::default();
    config.paths = Some(Paths {
        code: Some("code/".to_string()),
        docs: Some("docs/".to_string()),
        files: Some(vec![file_name]),
    });

    match style {
        "md" => config.md = Some(MdParser::default()),
        "tex" => config.tex = Some(TexParser::default()),
        "html" => config.html = Some(HtmlParser::default()),
        "bird" => config.bird = Some(BirdParser::default()),
        _ => {}
    };

    let toml = toml::to_string(&config).unwrap();
    let mut toml_file = File::create(&toml_path)?;
    toml_file.write_all(&toml.as_bytes())?;

    let doc = "".to_string();
    let mut base_file = File::create(&base_path)?;
    base_file.write_all(&doc.as_bytes())?;

    Ok(())
}

fn compile<P>(
    parser: &P,
    source: &str,
    doc_dir: &Either<(), Option<PathBuf>>,
    code_dir: &Either<(), Option<PathBuf>>,
    file_name: &Option<PathBuf>,
    entrypoint: Option<&str>,
    language: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>>
where
    P: Parser + Printer,
    P::Error: 'static,
{
    let document = parser.parse(source)?;

    let mut entries = vec![(entrypoint, file_name.clone())];
    entries.extend(parser.get_entry_points(&document).iter().map(|(e, p)| {
        (
            Some(*e),
            Some(PathBuf::from(p.to_owned().to_owned() + ".temp")),
        )
    }));

    if file_name.is_none() {
        match doc_dir {
            Left(..) => {
                let docs = document.print_docs(parser);
                print!("{}", docs);
            }
            Right(..) => {}
        }
    }

    match doc_dir {
        Left(..) => {
            let documentation = document.print_docs(parser);
            print!("{}", documentation);
        }
        Right(Some(doc_dir)) => {
            if let Some(file_name) = &file_name {
                let documentation = document.print_docs(parser);
                let mut file_path = doc_dir.clone();
                file_path.push(file_name);
                fs::create_dir_all(file_path.parent().unwrap()).unwrap();
                let mut doc_file = File::create(file_path).unwrap();
                write!(doc_file, "{}", documentation).unwrap();
            }
        }
        _ => {}
    }

    for (entrypoint, file_name) in entries {
        if file_name.is_none() {
            match doc_dir {
                Left(..) => {}
                Right(..) => {
                    let code = document.print_code(entrypoint, language)?;
                    println!("{}", code);
                }
            }
        }

        match code_dir {
            Left(..) => {
                let code = document.print_code(entrypoint, language)?;
                println!("{}", code);
            }
            Right(Some(code_dir)) => {
                if let Some(file_name) = &file_name {
                    let mut file_path = code_dir.clone();
                    if let Some(par) = file_name.parent() {
                        file_path.push(par)
                    }
                    file_path.push(file_name.file_stem().unwrap());
                    if let Some(language) = language {
                        file_path.set_extension(language);
                    }
                    fs::create_dir_all(file_path.parent().unwrap()).unwrap();
                    let mut code_file = File::create(file_path).unwrap();
                    let code = document.print_code(entrypoint, language)?;
                    write!(code_file, "{}", code).unwrap();
                }
            }
            _ => {}
        }
    }

    Ok(())
}
