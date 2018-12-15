use std::fs::{self, File};
use std::io::{Read, Write, stdin};
use std::path::PathBuf;
use clap::{Arg, App};
use serde_derive::Deserialize;
use outline::parser::{Parser, Printer, BirdParser, MdParser, TexParser, HtmlParser};

#[derive(Deserialize, Default)]
struct AnyConfig {
    bird: Option<BirdParser>,
    md: Option<MdParser>,
    tex: Option<TexParser>,
    html: Option<HtmlParser>,
}

fn main() {
    let matches = App::new("Outline")
        .version("1.0")
        .author("Cameron Eldridge <cameldridge@gmail.com>")
        .about("Literate programming compiler")
        .arg(Arg::with_name("config")
            .short("c")
            .long("config")
            .value_name("FILE")
            .help("Sets the config file name")
            .takes_value(true)
            .default_value("Outline.toml")
        )
        .arg(Arg::with_name("style")
            .short("s")
            .long("style")
            .value_name("STYLE")
            .help("Sets the style to use. If not specified, it is inferred from the file extension")
            .takes_value(true)
            .possible_values(&["bird", "md", "tex", "html"])
        )
        .arg(Arg::with_name("doc_dir")
            .short("d")
            .long("docs")
            .value_name("DOC_DIR")
            .help("Output documentation files to this directory")
            .takes_value(true)
        )
        .arg(Arg::with_name("code_dir")
            .short("o")
            .long("output")
            .value_name("OUTPUT_DIR")
            .help("Output code files to this directory")
            .takes_value(true)
        )
        .arg(Arg::with_name("entrypoint")
            .short("e")
            .long("entrypoint")
            .value_name("ENTRYPOINT")
            .help("The named entrypoint to use when tangling code")
            .takes_value(true)
        )
        .arg(Arg::with_name("language")
            .short("l")
            .long("language")
            .value_name("LANGUAGE")
            .help("The language to output the tangled code in. Only code blocks in this language will be used.")
            .takes_value(true)
        )
        .arg(Arg::with_name("input")
            .short("The input source file(s)")
            .value_name("INPUT")
            .multiple(true)
            .index(1)
        )
        .get_matches();

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
                    }
                    Err(error) => {
                        eprintln!("Could not read config file \"{}\": {}", file_name, error);
                        return;
                    }
                }
            }
        }
    };

    let doc_dir = matches.value_of("doc_dir").map(PathBuf::from);
    let code_dir = matches.value_of("code_dir").map(PathBuf::from);

    enum Input<'a> { File(&'a str), Stdin }
    let inputs = matches.values_of("input")
        .map(|files| files.into_iter().map(|file| Input::File(file)).collect())
        .unwrap_or(vec![Input::Stdin]);

    for input in inputs {
        let (file_name, contents, style_type, code_type) = match input {
            Input::File(file_name) => {
                let file_name = PathBuf::from(file_name);

                let contents = match fs::read_to_string(&file_name) {
                    Ok(contents) => contents,
                    Err(error) => {
                        eprintln!("Could not read source file \"{}\": {}", file_name.to_str().unwrap(), error);
                        return;
                    }
                };

                let style_type = file_name.extension()
                    .and_then(|osstr| osstr.to_str())
                    .map(|s| s.to_owned());

                let code_type = file_name.file_stem()
                    .and_then(|stem| PathBuf::from(stem)
                        .extension()
                        .and_then(|osstr| osstr.to_str())
                        .map(|s| s.to_owned())
                    );
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

        match matches.value_of("style").map(|s| s.to_string()).or(style_type).unwrap_or("md".to_string()).as_str() {
            "bird" => {
                let default = BirdParser::default();
                let parser = any_config.bird.as_ref().unwrap_or(&default);
                if let Err(error) = compile(parser, &contents, &doc_dir, &code_dir, &file_name, entrypoint, language) {
                    if let Some(file_name) = file_name {
                        eprintln!("Failed to compile source file \"{}\": {}", file_name.to_str().unwrap(), error);
                    } else {
                        eprintln!("Failed to compile from STDIN: {}", error);
                    }
                    continue;
                }
            }
            "md" => {
                let default = MdParser::default();
                let parser = any_config.md
                    .as_ref()
                    .unwrap_or(&default)
                    .default_language(code_type);
                if let Err(error) = compile(&parser, &contents, &doc_dir, &code_dir, &file_name, entrypoint, language) {
                    if let Some(file_name) = file_name {
                        eprintln!("Failed to compile source file \"{}\": {}", file_name.to_str().unwrap(), error);
                    } else {
                        eprintln!("Failed to compile from STDIN: {}", error);
                    }
                    continue;
                }
            }
            "tex" => {
                let default = TexParser::default();
                let parser = any_config.tex
                    .as_ref()
                    .unwrap_or(&default)
                    .default_language(code_type);
                if let Err(error) = compile(&parser, &contents, &doc_dir, &code_dir, &file_name, entrypoint, language) {
                    if let Some(file_name) = file_name {
                        eprintln!("Failed to compile source file \"{}\": {}", file_name.to_str().unwrap(), error);
                    } else {
                        eprintln!("Failed to compile from STDIN: {}", error);
                    }
                    continue;
                }
            }
            "html" => {
                let default = HtmlParser::default();
                let parser = any_config.html
                    .as_ref()
                    .unwrap_or(&default)
                    .default_language(code_type);
                if let Err(error) = compile(&parser, &contents, &doc_dir, &code_dir, &file_name, entrypoint, language) {
                    if let Some(file_name) = file_name {
                        eprintln!("Failed to compile source file \"{}\": {}", file_name.to_str().unwrap(), error);
                    } else {
                        eprintln!("Failed to compile from STDIN: {}", error);
                    }
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

fn compile<P>(
    parser: &P,
    source: &str,
    doc_dir: &Option<PathBuf>,
    code_dir: &Option<PathBuf>,
    file_name: &Option<PathBuf>,
    entrypoint: Option<&str>,
    language: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>>
where
    P: Parser + Printer,
    P::Error: 'static,
{
    let document = parser.parse(source)?;

    if file_name.is_none() {
        let code = document.print_code(entrypoint, language)?;
        print!("{}", code);
    }

    if let Some(code_dir) = code_dir {
        if let Some(file_name) = file_name {
            let mut file_path = code_dir.clone();
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

    if let Some(doc_dir) = doc_dir {
        let documentation = document.print_docs(parser);
        if let Some(file_name) = file_name {
            let mut file_path = doc_dir.clone();
            file_path.push(file_name);
            fs::create_dir_all(file_path.parent().unwrap()).unwrap();
            let mut doc_file = File::create(file_path).unwrap();
            write!(doc_file, "{}", documentation).unwrap();
        }
    }

    Ok(())
}
