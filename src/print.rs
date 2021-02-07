use crate::config::{LanguageSettings, ParserSettings};
use crate::document::code::{CodeBlock, Line, Source};
use crate::document::transclusion::Transclusion;
use crate::document::{CompileError, CompileErrorKind, Document, Node};
use crate::parser::code::RevCodeBlock;
use std::collections::HashMap;
use std::fmt::Write;

/// Formats this `Document` as a string containing the documentation file contents
pub fn print_docs(document: &Document, settings: &ParserSettings) -> String {
    let mut output = String::new();
    for node in &document.nodes {
        match node {
            Node::Transclusion(transclusion) => {
                print_transclusion(transclusion, false, &mut output)
            }
            Node::Text(text_block) => {
                writeln!(output, "{}", text_block).unwrap();
            }
            Node::Code(code_block) => {
                if !code_block.hidden {
                    print_code_block(code_block, settings, &code_block.indent, &mut output);
                }
            }
        }
    }
    output
}

/// Formats this `Document` as a string containing the compiled code
pub fn print_code(
    document: &Document,
    entrypoint: Option<&str>,
    language: Option<&str>,
    settings: Option<&LanguageSettings>,
) -> Result<String, CompileError> {
    let block_labels = settings.and_then(|s| s.block_labels.as_ref());
    let comment_start = block_labels
        .map(|l| l.comment_start.as_str())
        .unwrap_or_default();
    let comment_end = block_labels
        .and_then(|l| l.comment_end.as_deref())
        .unwrap_or_default();
    let block_start = block_labels
        .map(|l| l.block_start.as_str())
        .unwrap_or_default();
    let block_end = block_labels
        .map(|l| l.block_end.as_str())
        .unwrap_or_default();
    let block_next = block_labels
        .map(|l| l.block_next.as_str())
        .unwrap_or_default();
    let block_name_sep = '#';

    let clean = if let Some(s) = settings {
        s.clean_code || s.block_labels.is_none()
    } else {
        true
    };

    let code_blocks = document.code_blocks_by_name(language);
    let mut result = String::new();
    match code_blocks.get(&entrypoint) {
        Some(blocks) => {
            let mut block_count: HashMap<&Option<String>, usize> = HashMap::new();
            for (idx, block) in blocks.iter().enumerate() {
                let index = {
                    let count = block_count.entry(&block.name).or_default();
                    *count += 1;
                    *count - 1
                };

                let path = block.source_file.to_owned().unwrap_or_default();
                let name = if block.is_unnamed {
                    ""
                } else {
                    block.name.as_ref().map(|n| &n[..]).unwrap_or("")
                };

                if !clean {
                    let sep = if idx == 0 || block.name != blocks[idx - 1].name {
                        &block_start
                    } else {
                        &block_next
                    };
                    writeln!(
                        result,
                        "{} {}{}{}{}{}{}{}",
                        comment_start,
                        sep,
                        path,
                        block_name_sep,
                        name,
                        block_name_sep,
                        index,
                        comment_end,
                    )
                    .unwrap();
                }
                writeln!(result, "{}", block.compile(&code_blocks, settings)?).unwrap();

                if !clean && (idx == blocks.len() - 1 || block.name != blocks[idx + 1].name) {
                    write!(
                        result,
                        "{} {}{}{}{}{}{}{}",
                        comment_start,
                        block_end,
                        path,
                        block_name_sep,
                        name,
                        block_name_sep,
                        index,
                        comment_end,
                    )
                    .unwrap();
                }
            }
        }
        None => {
            return Err(CompileError::Single {
                line_number: 0,
                kind: CompileErrorKind::MissingEntrypoint,
            })
        }
    }
    if settings.map(|s| s.eof_newline).unwrap_or(true) && !result.ends_with('\n') {
        writeln!(result).unwrap();
    }
    Ok(result)
}

/// Formats this `Document` as the original source, potentially replacing code blocks
pub fn print_reverse(
    document: &Document,
    settings: &ParserSettings,
    code_blocks: &HashMap<(&Option<String>, &usize), &RevCodeBlock>,
) -> String {
    let mut block_count: HashMap<&Option<String>, usize> = HashMap::new();

    let mut output = String::new();
    for node in &document.nodes {
        match node {
            Node::Transclusion(transclusion) => print_transclusion(transclusion, true, &mut output),
            Node::Text(text_block) => {
                writeln!(output, "{}", text_block).unwrap();
            }
            Node::Code(code_block) => {
                let index = {
                    let count = block_count.entry(&code_block.name).or_default();
                    *count += 1;
                    *count - 1
                };

                let alt_block = code_blocks.get(&(&code_block.name, &index));

                print_code_block_reverse(
                    code_block,
                    alt_block.copied(),
                    settings,
                    &code_block.indent,
                    &mut output,
                );
            }
        }
    }
    output
}

pub fn print_transclusion(transclusion: &Transclusion, reverse: bool, write: &mut impl Write) {
    if reverse {
        writeln!(write, "{}", transclusion.original()).unwrap();
    } else {
        write!(write, "**WARNING!** Missed/skipped transclusion: ").unwrap();
        writeln!(write, "{}", transclusion.file().to_str().unwrap()).unwrap();
    }
}

pub fn print_code_block(
    block: &CodeBlock,
    settings: &ParserSettings,
    indent: &str,
    write: &mut impl Write,
) {
    let fence_sequence = if block.alternative {
        &settings.fence_sequence_alt
    } else {
        &settings.fence_sequence
    };
    write!(write, "{}{}", indent, fence_sequence).unwrap();
    if let Some(language) = &block.language {
        write!(write, "{}", language).unwrap();
    }
    writeln!(write).unwrap();
    if let Some(name) = &block.name {
        writeln!(write, "{}{} {}", indent, settings.block_name_prefix, name).unwrap();
    }

    let mut comments = vec![];
    let line_offset = block.line_number().unwrap_or(0);
    for line in &block.source {
        print_line(&line, settings, !settings.comments_as_aside, indent, write);
        if settings.comments_as_aside {
            if let Some(comment) = &line.comment {
                comments.push((line.line_number - line_offset, comment));
            }
        }
    }

    writeln!(write, "{}{}", indent, fence_sequence).unwrap();

    for (line, comment) in comments {
        writeln!(
            write,
            "<aside class=\"comment\" data-line=\"{}\">{}</aside>",
            line,
            comment.trim()
        )
        .unwrap();
    }
}

pub fn print_code_block_reverse(
    block: &CodeBlock,
    alternative: Option<&RevCodeBlock>,
    settings: &ParserSettings,
    indent: &str,
    write: &mut impl Write,
) {
    let fence_sequence = if block.alternative {
        &settings.fence_sequence_alt
    } else {
        &settings.fence_sequence
    };
    write!(write, "{}{}", indent, fence_sequence).unwrap();
    if let Some(language) = &block.language {
        write!(write, "{}", language).unwrap();
    }
    writeln!(write).unwrap();
    if let Some(name) = &block.name {
        write!(write, "{}{} ", indent, settings.block_name_prefix).unwrap();
        if block.hidden {
            write!(write, "{}", settings.hidden_prefix).unwrap();
        }
        writeln!(write, "{}", name).unwrap();
    }

    if let Some(alt) = alternative {
        for line in &alt.lines {
            if line.is_empty() {
                writeln!(write).unwrap();
            } else {
                writeln!(write, "{}{}", indent, line).unwrap();
            }
        }
    } else {
        for line in &block.source {
            // TODO: check if it is necessary to clear empty lines
            print_line(&line, settings, true, indent, write);
        }
    }
    writeln!(write, "{}", fence_sequence).unwrap();
}

/// Prints a line of a code block
pub fn print_line(
    line: &Line,
    settings: &ParserSettings,
    print_comments: bool,
    indent: &str,
    write: &mut impl Write,
) {
    write!(write, "{}{}", indent, line.indent).unwrap();
    match &line.source {
        Source::Macro(name) => {
            write!(write, "{}", settings.macro_start).unwrap();
            if !settings.macro_start.ends_with(' ') {
                write!(write, " ").unwrap();
            }
            write!(write, "{}{}", name, settings.macro_end).unwrap();
        }
        Source::Source(string) => {
            write!(write, "{}", string).unwrap();
        }
    }
    if print_comments {
        if let Some(comment) = &line.comment {
            write!(write, "{}{}", settings.block_name_prefix, comment).unwrap();
        }
    }
    writeln!(write).unwrap();
}
