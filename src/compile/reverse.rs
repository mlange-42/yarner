use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use yarner_lib::Document;

use crate::{config::Config, files, parse, util::Fallible};

pub fn compile_all(
    config: &Config,
    file_name: &Path,
    track_input_files: &mut HashSet<PathBuf>,
    track_code_files: &mut HashSet<PathBuf>,
    documents: &mut HashMap<PathBuf, Document>,
) -> Fallible {
    if !track_input_files.contains(file_name) {
        let mut trace = HashSet::new();
        let (mut document, links) = transclude_dry_run(
            config,
            file_name,
            file_name,
            documents,
            track_input_files,
            track_code_files,
            &mut trace,
        )?;

        let file_str = file_name.to_str().unwrap();
        super::set_source(&mut document, file_str);

        compile(config, &document, file_name, track_code_files);

        documents.insert(file_name.to_owned(), document);

        track_input_files.insert(file_name.to_owned());

        for file in links {
            if file.is_file() {
                if !track_input_files.contains(&file) {
                    compile_all(
                        config,
                        &file,
                        track_input_files,
                        track_code_files,
                        documents,
                    )?;
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
    track_code_files: &mut HashSet<PathBuf>,
) {
    println!("Compiling file {}", file_name.display());

    let mut entries = document.entry_points();

    let file_name_without_ext = file_name.with_extension("");
    entries.insert(
        config.paths.entrypoint.as_deref(),
        (&file_name_without_ext, Some(PathBuf::from(file_name))),
    );

    for (entrypoint, (sub_file_name, _sub_source_file)) in entries {
        if document.code_blocks_by_name().contains_key(&entrypoint) {
            let mut file_path = config.paths.code.clone().unwrap_or_default();
            file_path.push(sub_file_name);

            track_code_files.insert(file_path);
        }
    }
}

fn transclude_dry_run(
    config: &Config,
    root_file: &Path,
    file_name: &Path,
    documents: &mut HashMap<PathBuf, Document>,
    source_files: &mut HashSet<PathBuf>,
    track_code_files: &mut HashSet<PathBuf>,
    trace: &mut HashSet<PathBuf>,
) -> Fallible<(Document, Vec<PathBuf>)> {
    if trace.contains(file_name) {
        return Err(format!(
            "Circular transclusion: {} (root: {})",
            file_name.display(),
            root_file.display()
        )
        .into());
    } else {
        trace.insert(file_name.to_owned());
    }

    let source_main = files::read_file_string(&file_name)?;
    let (document, mut links) =
        parse::parse(&source_main, &root_file, &file_name, true, &config.parser)?;

    let transclusions = document.transclusions();

    let mut trans_so_far = HashSet::new();
    for trans in transclusions {
        if !trans_so_far.contains(&trans.file) {
            let (doc, sub_links) = transclude_dry_run(
                config,
                root_file,
                &trans.file,
                documents,
                source_files,
                track_code_files,
                trace,
            )?;
            source_files.insert(trans.file.to_owned());

            if doc.newline() != document.newline() {
                return Err(format!(
                    "Different EndOfLine sequences used in files {} and {}.\n  Change line endings of one of the files and try again.",
                    file_name.display(),
                    trans.file.display(),
                )
                    .into());
            }

            compile(config, &doc, &trans.file, track_code_files);

            links.extend(sub_links.into_iter());
            documents.insert(trans.file.clone(), doc);
            trans_so_far.insert(trans.file.clone());
        } else {
            return Err(format!("Multiple transclusions of {}", trans.file.display()).into());
        }
    }

    Ok((document, links))
}
