use std::collections::{
    hash_map::Entry::{Occupied, Vacant},
    HashMap, HashSet,
};
use std::path::{Path, PathBuf};

use crate::{
    code,
    code::RevCodeBlock,
    config::Config,
    config::{LanguageSettings, ParserSettings},
    document::Document,
    files, parse,
    util::Fallible,
};

type BlockKey = (PathBuf, Option<String>, usize);

#[allow(clippy::too_many_arguments)]
pub fn compile_all(
    parser: &ParserSettings,
    doc_dir: Option<&Path>,
    code_dir: &Path,
    file_name: &Path,
    entrypoint: Option<&str>,
    language: Option<&str>,
    settings: &HashMap<String, LanguageSettings>,
    track_input_files: &mut HashSet<PathBuf>,
    track_code_files: &mut HashSet<PathBuf>,
    documents: &mut HashMap<PathBuf, Document>,
) -> Fallible {
    if !track_input_files.contains(file_name) {
        let (mut document, links) = transclude_dry_run(
            parser,
            file_name,
            code_dir,
            entrypoint,
            language,
            documents,
            track_code_files,
        )?;

        let file_str = file_name.to_str().unwrap();
        document.set_source(file_str);

        compile(
            parser,
            &document,
            code_dir,
            file_name,
            entrypoint,
            language,
            track_code_files,
        )?;

        documents.insert(file_name.to_owned(), document);

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
                    documents,
                )?;
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

#[allow(clippy::too_many_arguments)]
fn compile(
    parser: &ParserSettings,
    document: &Document,
    code_dir: &Path,
    file_name: &Path,
    entrypoint: Option<&str>,
    language: Option<&str>,
    track_code_files: &mut HashSet<PathBuf>,
) -> Fallible {
    println!("Compiling file {}", file_name.display());

    let mut entries = document.entry_points(parser, language);

    let file_name_without_ext = file_name.with_extension("");
    entries.insert(
        entrypoint,
        (&file_name_without_ext, Some(PathBuf::from(file_name))),
    );

    for (_entrypoint, (sub_file_name, _sub_source_file)) in entries {
        let mut file_path = code_dir.to_owned();
        file_path.push(sub_file_name);
        if let Some(language) = language {
            file_path.set_extension(language);
        }

        track_code_files.insert(file_path);
    }

    Ok(())
}

fn transclude_dry_run(
    parser: &ParserSettings,
    file_name: &Path,
    code_dir: &Path,
    entrypoint: Option<&str>,
    language: Option<&str>,
    documents: &mut HashMap<PathBuf, Document>,
    track_code_files: &mut HashSet<PathBuf>,
) -> Fallible<(Document, Vec<PathBuf>)> {
    let source_main = files::read_file_string(&file_name)?;
    let (document, mut links) = parse::parse(&source_main, &file_name, true, parser)?;

    let transclusions = document.transclusions();

    let mut trans_so_far = HashSet::new();
    for trans in transclusions {
        if !trans_so_far.contains(trans.file()) {
            let (doc, sub_links) = transclude_dry_run(
                parser,
                trans.file(),
                code_dir,
                entrypoint,
                language,
                documents,
                track_code_files,
            )?;

            compile(
                parser,
                &doc,
                code_dir,
                trans.file(),
                entrypoint,
                language,
                track_code_files,
            )?;

            links.extend(sub_links.into_iter());
            documents.insert(trans.file().clone(), doc);
            trans_so_far.insert(trans.file().clone());
        } else {
            return Err(format!("Multiple transclusions of {}", trans.file().display()).into());
        }
    }

    Ok((document, links))
}
