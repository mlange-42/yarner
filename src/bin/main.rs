use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use clap::{Arg, App};
use outline::parser::{Parser, BirdParser};

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

    let config = match matches.value_of("config") {
        None => BirdParser::default(),
        Some(file_name) => {
            match fs::read_to_string(file_name) {
                Ok(string) => match toml::from_str(&string) {
                    Ok(config) => config,
                    Err(error) => {
                        eprintln!("{}", error);
                        return;
                    }
                },
                Err(error) => {
                    eprintln!("{}", error);
                    return;
                }
            }
        }
    };

    let doc_dir = matches.value_of("doc_dir").map(PathBuf::from);
    let code_dir = matches.value_of("code_dir").map(PathBuf::from);

    if let Some(values) = matches.values_of("input") {
        for file_name in values {
            let contents = match fs::read_to_string(file_name) {
                Ok(contents) => contents,
                Err(error) => {
                    eprintln!("{}", error);
                    return;
                }
            };

            match config.parse(&contents) {
                Ok(document) => {
                    if let Some(code_dir) = &code_dir {
                        let mut file_path = code_dir.clone();
                        file_path.push(PathBuf::from(file_name).file_stem().unwrap());
                        fs::create_dir_all(file_path.parent().unwrap()).unwrap();
                        let mut code_file = File::create(file_path).unwrap();
                        match document.print_code() {
                            Ok(output) => writeln!(code_file, "{}", output).unwrap(),
                            Err(error) => {
                                eprintln!("{}", error);
                                return;
                            }
                        };
                    }

                    if let Some(doc_dir) = &doc_dir {
                        let mut file_path = doc_dir.clone();
                        file_path.push(file_name);
                        fs::create_dir_all(file_path.parent().unwrap()).unwrap();
                        let mut doc_file = File::create(file_path).unwrap();
                        writeln!(doc_file, "{}", document.print_docs(&config)).unwrap();
                    }
                }
                Err(error) => {
                    eprintln!("{}", error);
                    return;
                }
            }
        }
    } else {
        eprintln!("Not yet supported");
    }
}
