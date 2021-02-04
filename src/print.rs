use crate::code::RevCodeBlock;
use crate::config::{LanguageSettings, ParserSettings};
use crate::document::code::{CodeBlock, Line, Source};
use crate::document::transclusion::Transclusion;
use crate::document::{CompileError, CompileErrorKind, Document, Node};
use std::collections::HashMap;
use std::fmt::Write;

/// Formats this `Document` as a string containing the documentation file contents
pub fn print_docs(document: &Document, settings: &ParserSettings) -> String {
    let mut output = String::new();
    for node in &document.nodes {
        match node {
            Node::Transclusion(transclusion) => {
                output.push_str(&print_transclusion(transclusion, false))
            }
            Node::Text(text_block) => {
                output.push_str(&text_block.to_string());
                output.push('\n')
            }
            Node::Code(code_block) => {
                if !code_block.hidden {
                    output.push_str(
                        &print_code_block(code_block, settings)
                            .split('\n')
                            .map(|line| {
                                if line.is_empty() {
                                    line.to_string()
                                } else {
                                    format!("{}{}", code_block.indent, line)
                                }
                            })
                            .collect::<Vec<_>>()
                            .join("\n"),
                    )
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
    let comment_start = settings
        .and_then(|s| s.block_labels.as_ref())
        .map(|l| l.comment_start.as_str())
        .unwrap_or_default();
    let comment_end = settings
        .and_then(|s| s.block_labels.as_ref())
        .and_then(|l| l.comment_end.as_deref())
        .unwrap_or_default();
    let block_start = settings
        .and_then(|s| s.block_labels.as_ref())
        .map(|l| l.block_start.as_str())
        .unwrap_or_default();
    let block_end = settings
        .and_then(|s| s.block_labels.as_ref())
        .map(|l| l.block_end.as_str())
        .unwrap_or_default();
    let block_next = settings
        .and_then(|s| s.block_labels.as_ref())
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
                result.push_str(&block.compile(&code_blocks, settings)?);
                result.push('\n');
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
        result.push('\n');
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
            Node::Transclusion(transclusion) => {
                output.push_str(&print_transclusion(transclusion, true))
            }
            Node::Text(text_block) => {
                output.push_str(&text_block.to_string());
                output.push('\n');
            }
            Node::Code(code_block) => {
                let index = {
                    let count = block_count.entry(&code_block.name).or_default();
                    *count += 1;
                    *count - 1
                };

                let alt_block = code_blocks.get(&(&code_block.name, &index));

                output.push_str(
                    &print_code_block_reverse(code_block, alt_block.copied(), settings)
                        .split('\n')
                        .map(|line| {
                            if line.is_empty() {
                                line.to_string()
                            } else {
                                format!("{}{}", code_block.indent, line)
                            }
                        })
                        .collect::<Vec<_>>()
                        .join("\n"),
                )
            }
        }
    }
    output
}

pub fn print_transclusion(transclusion: &Transclusion, reverse: bool) -> String {
    let mut output = String::new();
    if reverse {
        output.push_str(transclusion.original());
        output.push('\n');
    } else {
        output.push_str("**WARNING!** Missed/skipped transclusion: ");
        output.push_str(transclusion.file().to_str().unwrap());
        output.push('\n');
    }
    output
}

pub fn print_code_block(block: &CodeBlock, settings: &ParserSettings) -> String {
    let fence_sequence = if block.alternative {
        &settings.fence_sequence_alt
    } else {
        &settings.fence_sequence
    };
    let mut output = fence_sequence.clone();
    if let Some(language) = &block.language {
        output.push_str(language);
    }
    output.push('\n');
    if let Some(name) = &block.name {
        output.push_str(&settings.block_name_prefix);
        output.push(' ');
        output.push_str(name);
        output.push('\n');
    }

    let mut comments = vec![];
    let line_offset = block.line_number().unwrap_or(0);
    for line in &block.source {
        output.push_str(&print_line(&line, settings, !settings.comments_as_aside));
        if settings.comments_as_aside {
            if let Some(comment) = &line.comment {
                comments.push((line.line_number - line_offset, comment));
            }
        }
        output.push('\n');
    }

    output.push_str(fence_sequence);
    output.push('\n');

    for (line, comment) in comments {
        output.push_str(&format!(
            "<aside class=\"comment\" data-line=\"{}\">{}</aside>\n",
            line,
            comment.trim()
        ));
    }

    output
}

pub fn print_code_block_reverse(
    block: &CodeBlock,
    alternative: Option<&RevCodeBlock>,
    settings: &ParserSettings,
) -> String {
    let fence_sequence = if block.alternative {
        &settings.fence_sequence_alt
    } else {
        &settings.fence_sequence
    };
    let mut output = fence_sequence.clone();
    if let Some(language) = &block.language {
        output.push_str(language);
    }
    output.push('\n');
    if let Some(name) = &block.name {
        output.push_str(&settings.block_name_prefix);
        output.push(' ');
        if block.hidden {
            output.push_str(&settings.hidden_prefix);
        }
        output.push_str(name);
        output.push('\n');
    }

    if let Some(alt) = alternative {
        for line in &alt.lines {
            output.push_str(&line);
            output.push('\n');
        }
    } else {
        for line in &block.source {
            output.push_str(&print_line(&line, settings, true));
            output.push('\n');
        }
    }

    output.push_str(fence_sequence);
    output.push('\n');

    output
}

/// Prints a line of a code block
pub fn print_line(line: &Line, settings: &ParserSettings, print_comments: bool) -> String {
    let mut output = line.indent.to_string();
    match &line.source {
        Source::Macro(name) => {
            output.push_str(&settings.macro_start);
            if !settings.macro_start.ends_with(' ') {
                output.push(' ');
            }
            output.push_str(name);
            output.push_str(&settings.macro_end);
        }
        Source::Source(string) => {
            output.push_str(string);
        }
    }
    if print_comments {
        if let Some(comment) = &line.comment {
            output.push_str(&settings.block_name_prefix);
            output.push_str(&comment);
        }
    }
    output
}
