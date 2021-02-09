use std::collections::{
    hash_map::Entry::{Occupied, Vacant},
    HashMap, HashSet,
};
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::{
    config::{LanguageSettings, ParserSettings},
    document::{CompileError, CompileErrorKind, Document},
    files, parse, print,
    util::Fallible,
};

#[allow(clippy::too_many_arguments)]
pub fn compile_all(
    parser: &ParserSettings,
    doc_dir: Option<&Path>,
    code_dir: Option<&Path>,
    file_name: &Path,
    entrypoint: Option<&str>,
    language: Option<&str>,
    settings: &HashMap<String, LanguageSettings>,
    track_input_files: &mut HashSet<PathBuf>,
    track_code_files: &mut HashMap<PathBuf, Option<PathBuf>>,
) -> Fallible {
    if !track_input_files.contains(file_name) {
        let (mut document, links) = transclude(parser, file_name)?;

        let file_str = file_name.to_str().unwrap();
        document.set_source(file_str);

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
fn compile(
    parser: &ParserSettings,
    document: &Document,
    doc_dir: Option<&Path>,
    code_dir: Option<&Path>,
    file_name: &Path,
    entrypoint: Option<&str>,
    language: Option<&str>,
    settings: &HashMap<String, LanguageSettings>,
    track_code_files: &mut HashMap<PathBuf, Option<PathBuf>>,
) -> Fallible {
    eprintln!("Compiling file {}", file_name.display());

    let mut entries = document.entry_points(parser, language);

    let file_name_without_ext = file_name.with_extension("");
    entries.insert(
        entrypoint,
        (&file_name_without_ext, Some(file_name.to_owned())),
    );

    match doc_dir {
        Some(doc_dir) => {
            let documentation = print::print_docs(document, parser);
            let mut file_path = doc_dir.to_owned();
            file_path.push(file_name);
            fs::create_dir_all(file_path.parent().unwrap()).unwrap();
            let mut doc_file = File::create(file_path).unwrap();
            write!(doc_file, "{}", documentation).unwrap();
        }
        None => eprintln!("WARNING: Missing output location for docs, skipping docs output."),
    }

    for (entrypoint, (sub_file_name, sub_source_file)) in entries {
        match code_dir {
            Some(code_dir) => {
                let mut file_path = code_dir.to_owned();
                file_path.push(sub_file_name);
                if let Some(language) = language {
                    file_path.set_extension(language);
                }

                let extension = file_path
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .unwrap_or("")
                    .to_string();
                let settings = settings.get(&extension);

                match track_code_files.entry(file_path.clone()) {
                    Occupied(entry) => {
                        if sub_source_file == *entry.get() {
                            eprintln!("  Skipping file {} (already written)", file_path.display());
                            continue;
                        } else {
                            return Err(format!(
                                "Multiple distinct locations point to code file {}",
                                file_path.display()
                            )
                            .into());
                        }
                    }
                    Vacant(entry) => {
                        entry.insert(sub_source_file);
                    }
                }

                match print::print_code(document, entrypoint, language, settings) {
                    Ok(code) => {
                        eprintln!("  Writing file {}", file_path.display());
                        fs::create_dir_all(file_path.parent().unwrap()).unwrap();
                        let mut code_file = File::create(file_path).unwrap();
                        write!(code_file, "{}", code).unwrap()
                    }
                    Err(CompileError::Single {
                        kind: CompileErrorKind::MissingEntrypoint,
                        ..
                    }) => {
                        eprintln!(
                            "  WARNING: No entrypoint for file {}, skipping code output.",
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

fn transclude(parser: &ParserSettings, file_name: &Path) -> Fallible<(Document, Vec<PathBuf>)> {
    let source_main = files::read_file(&file_name)?;
    let (mut document, mut links) = parse::parse(&source_main, &file_name, false, parser)?;

    let transclusions = document.transclusions().cloned().collect::<Vec<_>>();

    let mut trans_so_far = HashSet::new();
    for trans in transclusions {
        if !trans_so_far.contains(trans.file()) {
            let (doc, sub_links) = transclude(parser, trans.file())?;

            // TODO: handle unwrap as error
            let ext = trans.file().extension().unwrap().to_str().unwrap();
            let full_path = trans.file().to_str().unwrap();
            let path = format!(
                "{}{}",
                parser.file_prefix,
                &full_path[..full_path.len() - ext.len() - 1]
            );
            document.transclude(&trans, doc, &full_path, &path[..]);

            links.extend(sub_links.into_iter());
            trans_so_far.insert(trans.file().clone());
        } else {
            return Err(format!("Multiple transclusions of {}", trans.file().display()).into());
        }
    }
    Ok((document, links))
}
