use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use clap::{Arg, App};
use serde_derive::Deserialize;
use outline::parser::{Parser, Printer, BirdParser, MdParser, TexParser};

#[derive(Deserialize, Default)]
struct AnyConfig {
    bird: Option<BirdParser>,
    md: Option<MdParser>,
    tex: Option<TexParser>,
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
            .help("Sets a custom config file")
            .takes_value(true)
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
        .arg(Arg::with_name("input")
            .short("The input source file")
            .value_name("INPUT")
            .multiple(true)
            .index(1)
        )
        .get_matches();

    let any_config: AnyConfig = match matches.value_of("config") {
        None => AnyConfig::default(),
        Some(file_name) => match fs::read_to_string(file_name) {
            Ok(config) => match toml::from_str(&config) {
                Ok(config) => config,
                Err(error) => {
                    eprintln!("{}", error);
                    return;
                }
            }
            Err(error) => {
                eprintln!("{}", error);
                return;
            }
        }
    };

    let doc_dir = matches.value_of("doc_dir").map(PathBuf::from);
    let code_dir = matches.value_of("code_dir").map(PathBuf::from);

    if let Some(values) = matches.values_of("input") {
        for file_name in values {
            let file_name = PathBuf::from(file_name);

            let contents = match fs::read_to_string(&file_name) {
                Ok(contents) => contents,
                Err(error) => {
                    eprintln!("{}", error);
                    return;
                }
            };

            let style_type = file_name.extension()
                .and_then(|osstr| osstr.to_str());

            let code_type = file_name.file_stem()
                .and_then(|stem| PathBuf::from(stem)
                    .extension()
                    .and_then(|osstr| osstr.to_str())
                    .map(|s| s.to_owned())
                );

            match matches.value_of("style").or(style_type).unwrap_or("md") {
                "bird" => {
                    let default = BirdParser::default();
                    let parser = any_config.bird.as_ref().unwrap_or(&default);
                    if let Err(error) = compile(parser, &contents, &doc_dir, &code_dir, &file_name) {
                        eprintln!("{}", error);
                        return;
                    }
                }
                "md" => {
                    let default = if let Some(language) = code_type {
                        MdParser::for_language(language.to_owned())
                    } else {
                        MdParser::default()
                    };
                    let parser = any_config.md
                        .as_ref()
                        .unwrap_or(&default);
                    if let Err(error) = compile(parser, &contents, &doc_dir, &code_dir, &file_name) {
                        eprintln!("{}", error);
                        return;
                    }
                }
                "tex" => {
                    let default = if let Some(language) = code_type {
                        TexParser::for_language(language.to_owned())
                    } else {
                        TexParser::default()
                    };
                    let parser = any_config.tex
                        .as_ref()
                        .unwrap_or(&default);
                    if let Err(error) = compile(parser, &contents, &doc_dir, &code_dir, &file_name) {
                        eprintln!("{}", error);
                        return;
                    }
                }
                other => {
                    eprintln!("Unknown style {}", other);
                    return;
                }
            };
        }
    } else {
        eprintln!("Not yet supported");
    }
}

fn compile<P>(
    parser: &P,
    source: &str,
    doc_dir: &Option<PathBuf>,
    code_dir: &Option<PathBuf>,
    file_name: &PathBuf
) -> Result<(), Box<dyn std::error::Error>>
where
    P: Parser + Printer,
    P::Error: 'static
{
    let document = parser.parse(source)?;

    if let Some(code_dir) = code_dir {
        let mut file_path = code_dir.clone();
        file_path.push(file_name.file_stem().unwrap());
        fs::create_dir_all(file_path.parent().unwrap()).unwrap();
        let mut code_file = File::create(file_path).unwrap();
        let code = document.print_code()?;
        writeln!(code_file, "{}", code).unwrap();
    }

    if let Some(doc_dir) = doc_dir {
        let mut file_path = doc_dir.clone();
        file_path.push(file_name);
        fs::create_dir_all(file_path.parent().unwrap()).unwrap();
        let mut doc_file = File::create(file_path).unwrap();
        writeln!(doc_file, "{}", document.print_docs(parser)).unwrap();
    }

    Ok(())
}
