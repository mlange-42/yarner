pub mod docs {
    use crate::code::RevCodeBlock;
    use crate::config::ParserSettings;
    use crate::util::JoinExt;
    use std::collections::HashMap;
    use std::fmt::Write;
    use yarner_lib::{CodeBlock, Document, Line, Node, Transclusion};

    /// Formats this `Document` as a string containing the documentation file contents
    pub fn print_docs(document: &Document, settings: &ParserSettings) -> String {
        let mut output = String::new();
        for node in &document.nodes {
            match node {
                Node::Transclusion(transclusion) => {
                    print_transclusion(transclusion, document.newline(), &mut output)
                }
                Node::Text(text_block) => {
                    write!(
                        output,
                        "{}{}",
                        text_block.text.iter().join(&document.newline(), ""),
                        document.newline()
                    )
                    .unwrap();
                }
                Node::Code(code_block) => {
                    if !code_block.is_hidden {
                        print_code_block(
                            code_block,
                            settings,
                            &code_block.indent,
                            document.newline(),
                            &mut output,
                        );
                    }
                }
            }
        }
        output
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
                    print_transclusion_reverse(transclusion, document.newline(), &mut output)
                }
                Node::Text(text_block) => {
                    write!(
                        output,
                        "{}{}",
                        text_block.text.iter().join(&document.newline(), ""),
                        document.newline()
                    )
                    .unwrap();
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
                        document.newline(),
                        &mut output,
                    );
                }
            }
        }
        output
    }

    fn print_transclusion(transclusion: &Transclusion, newline: &str, write: &mut impl Write) {
        write!(
            write,
            "**WARNING!** Missed/skipped transclusion: {}{}",
            transclusion.file.to_str().unwrap(),
            newline
        )
        .unwrap();
    }

    fn print_transclusion_reverse(
        transclusion: &Transclusion,
        newline: &str,
        write: &mut impl Write,
    ) {
        write!(write, "{}{}", transclusion.original, newline).unwrap();
    }

    fn print_code_block(
        block: &CodeBlock,
        settings: &ParserSettings,
        indent: &str,
        newline: &str,
        write: &mut impl Write,
    ) {
        let fence_sequence = if block.is_alternative {
            &settings.fence_sequence_alt
        } else {
            &settings.fence_sequence
        };
        write!(write, "{}{}", indent, fence_sequence).unwrap();
        if let Some(language) = &block.language {
            write!(write, "{}", language).unwrap();
        }
        write!(write, "{}", newline).unwrap();

        if let Some(name) = &block.name {
            write!(
                write,
                "{}{} {}{}{}",
                indent,
                settings.block_name_prefix,
                if block.is_file {
                    &settings.file_prefix
                } else {
                    ""
                },
                name,
                newline,
            )
            .unwrap();
        }

        for line in &block.source {
            print_line(line, settings, indent, newline, write);
        }

        write!(write, "{}{}{}", indent, fence_sequence, newline).unwrap();
    }

    fn print_code_block_reverse(
        block: &CodeBlock,
        alternative: Option<&RevCodeBlock>,
        settings: &ParserSettings,
        indent: &str,
        newline: &str,
        write: &mut impl Write,
    ) {
        let fence_sequence = if block.is_alternative {
            &settings.fence_sequence_alt
        } else {
            &settings.fence_sequence
        };
        write!(write, "{}{}", indent, fence_sequence).unwrap();
        if let Some(language) = &block.language {
            write!(write, "{}", language).unwrap();
        }
        write!(write, "{}", newline).unwrap();

        if let Some(name) = &block.name {
            write!(write, "{}{} ", indent, settings.block_name_prefix).unwrap();
            if block.is_hidden {
                write!(write, "{}", settings.hidden_prefix).unwrap();
            }
            if block.is_file {
                write!(write, "{}", settings.file_prefix).unwrap();
            }
            write!(write, "{}{}", name, newline).unwrap();
        }

        if let Some(alt) = alternative {
            for line in &alt.lines {
                if line.is_empty() {
                    write!(write, "{}", newline).unwrap();
                } else {
                    write!(write, "{}{}{}", indent, line, newline).unwrap();
                }
            }
        } else {
            for line in &block.source {
                print_line(line, settings, indent, newline, write);
            }
        }
        write!(write, "{}{}", fence_sequence, newline).unwrap();
    }

    /// Prints a line of a code block
    fn print_line(
        line: &Line,
        settings: &ParserSettings,
        block_indent: &str,
        newline: &str,
        write: &mut impl Write,
    ) {
        match line {
            Line::Macro { indent, name } => {
                write!(write, "{}{}{}", block_indent, indent, settings.macro_start).unwrap();
                if !settings.macro_start.ends_with(' ') {
                    write!(write, " ").unwrap();
                }
                write!(write, "{}{}", name, settings.macro_end).unwrap();
            }
            Line::Source { indent, source } => {
                write!(write, "{}{}{}", block_indent, indent, source).unwrap();
            }
        }
        write!(write, "{}", newline).unwrap();
    }

    #[cfg(test)]
    mod tests {
        use crate::config::Config;
        use yarner_lib::{CodeBlock, Line};

        #[test]
        fn print_code_block() {
            let config = toml::from_str::<Config>(include_str!("create/Yarner.toml")).unwrap();

            let code = CodeBlock {
                line_number: 1,
                indent: "".to_string(),
                name: Some("Code block".to_string()),
                is_unnamed: false,
                language: Some("rust".to_string()),
                is_file: false,
                is_hidden: false,
                is_alternative: false,
                source_file: None,
                source: vec![
                    Line::Source {
                        indent: "    ".to_string(),
                        source: "fn main() {}".to_string(),
                    },
                    Line::Macro {
                        indent: "    ".to_string(),
                        name: "Another block".to_string(),
                    },
                ],
            };

            let mut out = String::new();
            super::print_code_block(&code, &config.parser, "", "\n", &mut out);

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
}

pub mod code {
    use crate::config::LanguageSettings;
    use crate::util::{Fallible, JoinExt, TryCollectExt};
    use std::collections::{HashMap, HashSet};
    use std::fmt::Write;
    use yarner_lib::{CodeBlock, Line};

    /// Formats this `Document` as a string containing the compiled code
    pub fn print_code(
        code_blocks: &HashMap<Option<&str>, Vec<&CodeBlock>>,
        entry_blocks: &[&CodeBlock],
        settings: Option<&LanguageSettings>,
        newline: &str,
    ) -> Fallible<String> {
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

        let clean = settings.map_or(true, |set| set.clean_code || set.block_labels.is_none());

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
                    "{} {}{}{}{}{}{}{}{}",
                    comment_start,
                    sep,
                    path,
                    block_name_sep,
                    name,
                    block_name_sep,
                    index,
                    comment_end,
                    newline,
                )
                .unwrap();
            }

            let mut trace = HashSet::new();
            write!(
                result,
                "{}{}",
                compile_code_block(block, code_blocks, settings, newline, &mut trace)?
                    .join(newline, ""),
                newline,
            )
            .unwrap();

            if !clean && (idx == entry_blocks.len() - 1 || block.name != entry_blocks[idx + 1].name)
            {
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

        if settings.map(|s| s.eof_newline).unwrap_or(true) && !result.ends_with(newline) {
            write!(result, "{}", newline).unwrap();
        }
        Ok(result)
    }

    fn compile_code_block(
        block: &CodeBlock,
        code_blocks: &HashMap<Option<&str>, Vec<&CodeBlock>>,
        settings: Option<&LanguageSettings>,
        newline: &str,
        trace: &mut HashSet<String>,
    ) -> Result<Vec<String>, CompileError> {
        let line_offset = block.line_number;
        block
            .source
            .iter()
            .enumerate()
            .map(|(idx, line)| {
                compile_line(
                    line,
                    line_offset + if block.is_unnamed { idx } else { idx + 1 },
                    code_blocks,
                    settings,
                    newline,
                    trace,
                )
            })
            .try_collect()
            .map_err(CompileError::Multi)
    }

    fn compile_line(
        line: &Line,
        line_number: usize,
        code_blocks: &HashMap<Option<&str>, Vec<&CodeBlock>>,
        settings: Option<&LanguageSettings>,
        newline: &str,
        trace: &mut HashSet<String>,
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
        let block_next = block_labels.map(|l| l.block_next.as_str()).unwrap_or("");
        let block_name_sep = '#';

        let clean = if let Some(s) = settings {
            s.clean_code || s.block_labels.is_none()
        } else {
            true
        };

        let blank_lines = settings.map(|s| s.clear_blank_lines).unwrap_or(true);
        match line {
            Line::Source { indent, source } => {
                if blank_lines && source.trim().is_empty() {
                    Ok("".to_string())
                } else {
                    Ok(format!("{}{}", indent, source))
                }
            }
            Line::Macro { indent, name } => {
                if trace.contains(name) {
                    return Err(CompileError::Single {
                        line_number,
                        kind: CompileErrorKind::CircularReference(format!(
                            "Circular macro call: {}",
                            name,
                        )),
                    });
                } else {
                    trace.insert(name.clone());
                }

                let blocks = code_blocks.get(&Some(name)).ok_or(CompileError::Single {
                    line_number,
                    kind: CompileErrorKind::UnknownMacro(name.to_string()),
                })?;

                let mut result = String::new();
                for (idx, block) in blocks.iter().enumerate() {
                    let path = block.source_file.to_owned().unwrap_or_default();
                    let name = if block.is_unnamed {
                        ""
                    } else {
                        block.name.as_ref().map(|n| &n[..]).unwrap_or("")
                    };

                    if !clean {
                        write!(
                            result,
                            "{}{} {}{}{}{}{}{}{}{}",
                            indent,
                            comment_start,
                            if idx == 0 { &block_start } else { &block_next },
                            path,
                            block_name_sep,
                            name,
                            block_name_sep,
                            idx,
                            comment_end,
                            newline,
                        )
                        .unwrap();
                    }

                    let code = compile_code_block(block, code_blocks, settings, newline, trace)?;
                    for ln in code {
                        if blank_lines && ln.trim().is_empty() {
                            write!(result, "{}", newline).unwrap();
                        } else {
                            write!(result, "{}{}{}", indent, ln, newline).unwrap();
                        }
                    }

                    if !clean && idx == blocks.len() - 1 {
                        write!(
                            result,
                            "{}{} {}{}{}{}{}{}{}{}",
                            indent,
                            comment_start,
                            &block_end,
                            path,
                            block_name_sep,
                            name,
                            block_name_sep,
                            idx,
                            comment_end,
                            newline,
                        )
                        .unwrap();
                    }
                }
                for _ in 0..newline.len() {
                    result.pop();
                }
                Ok(result)
            }
        }
    }

    /// Problems encountered while compiling the document
    #[derive(Debug)]
    pub enum CompileErrorKind {
        /// An unknown macro name was encountered
        UnknownMacro(String),
        /// A macro results in a circular reference
        CircularReference(String),
    }

    /// Errors that were encountered while compiling the document
    #[derive(Debug)]
    pub enum CompileError {
        #[doc(hidden)]
        Multi(Vec<CompileError>),
        #[doc(hidden)]
        Single {
            line_number: usize,
            kind: CompileErrorKind,
        },
    }

    impl std::fmt::Display for CompileError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                CompileError::Multi(errors) => {
                    write!(f, "{}", errors.join("\n", ""))
                }
                CompileError::Single { line_number, kind } => {
                    write!(f, "{:?} (line {})", kind, line_number)
                }
            }
        }
    }

    impl std::error::Error for CompileError {}
}
