use std::collections::{
    hash_map::Entry::{Occupied, Vacant},
    HashMap, HashSet,
};
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

use yarner_lib::{Document, Node, Transclusion};

use crate::{
    config::{Config, ParserSettings},
    files, parse, print,
    util::Fallible,
};

pub fn compile_all(
    config: &Config,
    file_name: &Path,
    track_input_files: &mut HashSet<PathBuf>,
    track_code_files: &mut HashMap<PathBuf, Option<PathBuf>>,
) -> Fallible {
    if !track_input_files.contains(file_name) {
        let (mut document, links) = transclude(&config.parser, file_name, file_name)?;

        let file_str = file_name.to_str().unwrap();
        super::set_source(&mut document, file_str);

        compile(config, &document, file_name, track_code_files)?;
        track_input_files.insert(file_name.to_owned());

        for file in links {
            if file.is_file() {
                if !track_input_files.contains(&file) {
                    compile_all(config, &file, track_input_files, track_code_files)?;
                }
            } else {
                eprintln!("WARNING: link target not found for {}", file.display());
            }
        }
    }

    Ok(())
}

fn compile(
    config: &Config,
    document: &Document,
    file_name: &Path,
    track_code_files: &mut HashMap<PathBuf, Option<PathBuf>>,
) -> Fallible {
    println!("Compiling file {}", file_name.display());

    let mut entries = super::entry_points(document, &config.parser.file_prefix);

    let file_name_without_ext = file_name.with_extension("");
    entries.insert(
        config.paths.entrypoint.as_deref(),
        (&file_name_without_ext, Some(file_name.to_owned())),
    );

    match &config.paths.docs {
        Some(doc_dir) => {
            let documentation = print::docs::print_docs(document, &config.parser);
            let mut file_path = doc_dir.to_owned();
            file_path.push(file_name);
            fs::create_dir_all(file_path.parent().unwrap()).unwrap();
            let mut doc_file = File::create(file_path).unwrap();
            write!(doc_file, "{}", documentation).unwrap();
        }
        None => eprintln!("WARNING: Missing output location for docs, skipping docs output."),
    }

    for (entrypoint, (sub_file_name, sub_source_file)) in entries {
        match &config.paths.code {
            Some(code_dir) => {
                let code_blocks = document.code_blocks_by_name();
                if let Some(entry_blocks) = code_blocks.get(&entrypoint) {
                    let mut file_path = code_dir.to_owned();
                    file_path.push(sub_file_name);

                    let extension = file_path
                        .extension()
                        .and_then(|ext| ext.to_str())
                        .unwrap_or("")
                        .to_string();
                    let settings = config.language.get(&extension);

                    // TODO: only track files that are really created!
                    match track_code_files.entry(file_path.clone()) {
                        Occupied(entry) => {
                            if sub_source_file == *entry.get() {
                                println!(
                                    "  Skipping file {} (already written)",
                                    file_path.display()
                                );
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

                    let code = print::code::print_code(
                        &code_blocks,
                        entry_blocks,
                        settings,
                        document.newline(),
                    )?;
                    println!("  Writing file {}", file_path.display());
                    fs::create_dir_all(file_path.parent().unwrap())?;
                    let mut code_file = File::create(file_path)?;
                    write!(code_file, "{}", code)?;
                } else {
                    eprintln!(
                        "  No entrypoint for file {}, skipping code output.",
                        sub_file_name.display()
                    );
                }
            }
            None => eprintln!("WARNING: Missing output location for code, skipping code output."),
        }
    }

    Ok(())
}

fn transclude(
    parser: &ParserSettings,
    root_file: &Path,
    file_name: &Path,
) -> Fallible<(Document, Vec<PathBuf>)> {
    let source_main = files::read_file_string(&file_name)?;
    let (mut document, mut links) =
        parse::parse(&source_main, &root_file, &file_name, false, parser)?;

    let transclusions = document.transclusions().cloned().collect::<Vec<_>>();

    let mut trans_so_far = HashSet::new();
    for trans in transclusions {
        if !trans_so_far.contains(&trans.file) {
            let (doc, sub_links) = transclude(parser, root_file, &trans.file)?;

            if doc.newline() != document.newline() {
                return Err(format!(
                    "Different EndOfLine sequences used in files {} and {}.\n  Change line endings of one of the files and try again.",
                    file_name.display(),
                    trans.file.display(),
                )
                .into());
            }

            let path = format!(
                "{}{}",
                parser.file_prefix,
                trans.file.with_extension("").to_str().unwrap(),
            );
            transclude_into(&mut document, &trans, doc, &path);

            links.extend(sub_links.into_iter());
            trans_so_far.insert(trans.file.clone());
        } else {
            return Err(format!("Multiple transclusions of {}", trans.file.display()).into());
        }
    }
    Ok((document, links))
}

fn transclude_into(into: &mut Document, replace: &Transclusion, with: Document, from: &str) {
    let mut index = 0;
    while index < into.nodes.len() {
        if let Node::Transclusion(trans) = &into.nodes[index] {
            if trans == replace {
                into.nodes.remove(index);
                for (i, mut node) in with.nodes.into_iter().enumerate() {
                    if let Node::Code(code) = &mut node {
                        if code.name.is_none() {
                            code.name = Some(from.to_string());
                            code.is_unnamed = true;
                        }
                        if code.source_file.is_none() {
                            code.source_file = Some(replace.file.to_str().unwrap().to_owned());
                        }
                    };
                    into.nodes.insert(index + i, node);
                }
                // TODO: currently, only a single transclusion of a particular document is possible.
                // May be sufficient (or even desired), but should be checked.
                break;
            }
        }
        index += 1;
    }
}
