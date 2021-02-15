use std::collections::{
    hash_map::Entry::{Occupied, Vacant},
    HashMap, HashSet,
};
use std::path::{Path, PathBuf};

use crate::{
    code, code::RevCodeBlock, config::Config, document::Document, files, parse, util::Fallible,
};

type BlockKey = (PathBuf, Option<String>, usize);

pub fn compile_all(
    config: &Config,
    file_name: &Path,
    track_input_files: &mut HashSet<PathBuf>,
    track_code_files: &mut HashSet<PathBuf>,
    documents: &mut HashMap<PathBuf, Document>,
) -> Fallible {
    if !track_input_files.contains(file_name) {
        let (mut document, links) =
            transclude_dry_run(config, file_name, file_name, documents, track_code_files)?;

        let file_str = file_name.to_str().unwrap();
        document.set_source(file_str);

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

/// Collects all code blocks from the given code files
pub fn collect_code_blocks(
    code_files: &HashSet<PathBuf>,
    config: &Config,
) -> Fallible<HashMap<BlockKey, RevCodeBlock>> {
    let mut code_blocks: HashMap<BlockKey, RevCodeBlock> = HashMap::new();

    if !config.language.is_empty() {
        for file in code_files {
            let language = file.extension().and_then(|s| s.to_str());
            if let Some(language) = language {
                if let Some(labels) = config
                    .language
                    .get(language)
                    .and_then(|lang| lang.block_labels.as_ref())
                {
                    let source = files::read_file_string(&file)?;
                    let blocks = code::parse(&source, &config.parser, labels)?;

                    for block in blocks.into_iter() {
                        let path = PathBuf::from(&block.file);
                        match code_blocks.entry((path, block.name.clone(), block.index)) {
                            Occupied(entry) => {
                                if entry.get().lines != block.lines {
                                    return Err(format!("Reverse mode impossible due to multiple, differing occurrences of a code block: {} # {} # {}", &block.file, &block.name.unwrap_or_else(|| "".to_string()), block.index).into());
                                } else {
                                    eprintln!("  WARNING: multiple occurrences of a code block: {} # {} # {}", &block.file, &block.name.unwrap_or_else(|| "".to_string()), block.index)
                                }
                            }
                            Vacant(entry) => {
                                entry.insert(block);
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(code_blocks)
}

fn compile(
    config: &Config,
    document: &Document,
    file_name: &Path,
    track_code_files: &mut HashSet<PathBuf>,
) {
    println!("Compiling file {}", file_name.display());

    let mut entries = document.entry_points(&config.parser);

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
    track_code_files: &mut HashSet<PathBuf>,
) -> Fallible<(Document, Vec<PathBuf>)> {
    let source_main = files::read_file_string(&file_name)?;
    let (document, mut links) =
        parse::parse(&source_main, &root_file, &file_name, true, &config.parser)?;

    let transclusions = document.transclusions();

    let mut trans_so_far = HashSet::new();
    for trans in transclusions {
        if !trans_so_far.contains(trans.file()) {
            let (doc, sub_links) =
                transclude_dry_run(config, root_file, trans.file(), documents, track_code_files)?;

            compile(config, &doc, trans.file(), track_code_files);

            links.extend(sub_links.into_iter());
            documents.insert(trans.file().clone(), doc);
            trans_so_far.insert(trans.file().clone());
        } else {
            return Err(format!("Multiple transclusions of {}", trans.file().display()).into());
        }
    }

    Ok((document, links))
}
