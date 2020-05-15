use glob::glob;
use std::env;
use std::fs::{copy, create_dir_all, read_to_string, File};
use std::io::Write;
use std::path::PathBuf;
use yarner::parser::{MdParser, Parser};

fn main() {
    let out_dir: PathBuf = env::var("OUT_DIR").unwrap().into();

    for input_file in glob("./src/*/*.rs").unwrap() {
        let input_file = input_file.unwrap();
        let mut output_file = out_dir.clone();
        output_file.push(input_file.strip_prefix("src/").unwrap());
        create_dir_all(output_file.parent().unwrap()).unwrap();
        copy(input_file, output_file).unwrap();
    }

    for markdown in glob("./src/**/*.rs.md").unwrap() {
        let input_file_name = markdown.unwrap();
        let source = read_to_string(&input_file_name).unwrap();
        let result = MdParser::default()
            .parse(&source)
            .map_err(|error| format!("{}", error))
            .and_then(|document| {
                document
                    .print_code(None, Some("rs"))
                    .map_err(|error| format!("{}", error))
            });
        match result {
            Ok(code) => {
                let mut out_file = out_dir.clone();
                out_file.push(input_file_name.strip_prefix("src/").unwrap());
                out_file.set_file_name(input_file_name.file_stem().unwrap());
                create_dir_all(out_file.parent().unwrap()).unwrap();
                let mut file = File::create(out_file).unwrap();
                write!(file, "{}", code).unwrap();
            }
            Err(error) => {
                println!(
                    "cargo:warning=Failed to compile {:?}. Reason: {}",
                    input_file_name, error
                );
            }
        }
    }
}
