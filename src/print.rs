use crate::code::RevCodeBlock;
use crate::config::{LanguageSettings, ParserSettings};
use crate::document::{CodeBlock, CompileError, Document, Line, Node, Source, Transclusion};
use std::collections::HashMap;
use std::fmt::Write;

/// Formats this `Document` as a string containing the documentation file contents
pub fn print_docs(document: &Document, settings: &ParserSettings) -> String {
    let mut output = String::new();
    for node in &document.nodes {
        match node {
            Node::Transclusion(transclusion) => {
                print_transclusion(settings, transclusion, &mut output)
            }
            Node::Text(text_block) => {
                write!(output, "{}", text_block.lines().join(&settings.newline())).unwrap();
                write!(output, "{}", settings.newline()).unwrap();
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
    code_blocks: &HashMap<Option<&str>, Vec<&CodeBlock>>,
    entry_blocks: &[&CodeBlock],
    settings: &ParserSettings,
    language: Option<&LanguageSettings>,
) -> Result<String, CompileError> {
    let block_labels = language.and_then(|s| s.block_labels.as_ref());
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

    let clean = language.map_or(true, |set| set.clean_code || set.block_labels.is_none());

    let mut result = String::new();
    let mut block_count: HashMap<&Option<String>, usize> = HashMap::new();
    for (idx, block) in entry_blocks.iter().enumerate() {
        let index = {
            let count = block_count.entry(&block.name).or_default();
            *count += 1;
            *count - 1
        };

        let path = block.source_file.to_owned().unwrap_or_default();
        let name = if block.is_unnamed {
            ""
        } else {
            block.name.as_deref().unwrap_or("")
        };

        if !clean {
            let sep = if idx == 0 || block.name != entry_blocks[idx - 1].name {
                &block_start
            } else {
                &block_next
            };
            write!(
                result,
                "{} {}{}{}{}{}{}{}",
                comment_start, sep, path, block_name_sep, name, block_name_sep, index, comment_end,
            )
            .unwrap();
            write!(result, "{}", settings.newline()).unwrap();
        }
        write!(
            result,
            "{}",
            block.compile_with(&code_blocks, language, settings.newline())?
        )
        .unwrap();
        write!(result, "{}", settings.newline()).unwrap();

        if !clean && (idx == entry_blocks.len() - 1 || block.name != entry_blocks[idx + 1].name) {
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

    if language.map(|s| s.eof_newline).unwrap_or(true) && !result.ends_with(settings.newline()) {
        write!(result, "{}", settings.newline()).unwrap();
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
                print_transclusion_reverse(settings, transclusion, &mut output)
            }
            Node::Text(text_block) => {
                write!(output, "{}", text_block.lines().join(&settings.newline())).unwrap();
                write!(output, "{}", settings.newline()).unwrap();
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

pub fn print_transclusion(
    settings: &ParserSettings,
    transclusion: &Transclusion,
    write: &mut impl Write,
) {
    write!(write, "**WARNING!** Missed/skipped transclusion: ").unwrap();
    write!(write, "{}", transclusion.file().to_str().unwrap()).unwrap();
    write!(write, "{}", settings.newline()).unwrap();
}

pub fn print_transclusion_reverse(
    settings: &ParserSettings,
    transclusion: &Transclusion,
    write: &mut impl Write,
) {
    write!(write, "{}", transclusion.original()).unwrap();
    write!(write, "{}", settings.newline()).unwrap();
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
    write!(write, "{}", settings.newline()).unwrap();

    if let Some(name) = &block.name {
        write!(write, "{}{} {}", indent, settings.block_name_prefix, name).unwrap();
        write!(write, "{}", settings.newline()).unwrap();
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

    write!(write, "{}{}", indent, fence_sequence).unwrap();
    write!(write, "{}", settings.newline()).unwrap();

    for (line, comment) in comments {
        write!(
            write,
            "<aside class=\"comment\" data-line=\"{}\">{}</aside>",
            line,
            comment.trim()
        )
        .unwrap();
        write!(write, "{}", settings.newline()).unwrap();
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
    write!(write, "{}", settings.newline()).unwrap();

    if let Some(name) = &block.name {
        write!(write, "{}{} ", indent, settings.block_name_prefix).unwrap();
        if block.hidden {
            write!(write, "{}", settings.hidden_prefix).unwrap();
        }
        write!(write, "{}", name).unwrap();
        write!(write, "{}", settings.newline()).unwrap();
    }

    if let Some(alt) = alternative {
        for line in &alt.lines {
            if line.is_empty() {
                write!(write, "{}", settings.newline()).unwrap();
            } else {
                write!(write, "{}{}", indent, line).unwrap();
                write!(write, "{}", settings.newline()).unwrap();
            }
        }
    } else {
        for line in &block.source {
            // TODO: check if it is necessary to clear empty lines
            print_line(&line, settings, true, indent, write);
        }
    }
    write!(write, "{}", fence_sequence).unwrap();
    write!(write, "{}", settings.newline()).unwrap();
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
    write!(write, "{}", settings.newline()).unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    #[test]
    fn print_code_block() {
        let mut config = toml::from_str::<Config>(include_str!("create/Yarner.toml")).unwrap();
        config.parser.crlf_newline = Some(false);

        let code = CodeBlock {
            indent: "".to_string(),
            name: Some("Code block".to_string()),
            is_unnamed: false,
            language: Some("rust".to_string()),
            hidden: false,
            alternative: false,
            source_file: None,
            source: vec![
                Line {
                    line_number: 0,
                    indent: "    ".to_string(),
                    source: Source::Source("fn main() {}".to_string()),
                    comment: None,
                },
                Line {
                    line_number: 1,
                    indent: "    ".to_string(),
                    source: Source::Macro("Another block".to_string()),
                    comment: None,
                },
            ],
        };

        let mut out = String::new();
        super::print_code_block(&code, &config.parser, "", &mut out);

        assert_eq!(
            out,
            r#"```rust
//- Code block
    fn main() {}
    // ==> Another block.
```
"#
        )
    }
}
