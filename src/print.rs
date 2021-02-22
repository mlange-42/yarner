pub mod docs {
    use crate::code::RevCodeBlock;
    use crate::config::ParserSettings;
    use crate::document::{CodeBlock, Document, Line, Node, Source, Transclusion};
    use std::collections::HashMap;
    use std::fmt::Write;

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
                        text_block.text.join(&document.newline()),
                        document.newline()
                    )
                    .unwrap();
                }
                Node::Code(code_block) => {
                    if !code_block.hidden {
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
                        text_block.text.join(&document.newline()),
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
        let fence_sequence = if block.alternative {
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
                "{}{} {}{}",
                indent, settings.block_name_prefix, name, newline,
            )
            .unwrap();
        }

        let mut comments = vec![];
        for (line_number, line) in block.source.iter().enumerate() {
            print_line(
                &line,
                settings,
                !settings.comments_as_aside,
                indent,
                newline,
                write,
            );
            if settings.comments_as_aside {
                if let Some(comment) = &line.comment {
                    comments.push((line_number, comment));
                }
            }
        }

        write!(write, "{}{}{}", indent, fence_sequence, newline).unwrap();

        for (line, comment) in comments {
            write!(
                write,
                "<aside class=\"comment\" data-line=\"{}\">{}</aside>{}",
                line,
                comment.trim(),
                newline
            )
            .unwrap();
        }
    }

    fn print_code_block_reverse(
        block: &CodeBlock,
        alternative: Option<&RevCodeBlock>,
        settings: &ParserSettings,
        indent: &str,
        newline: &str,
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
        write!(write, "{}", newline).unwrap();

        if let Some(name) = &block.name {
            write!(write, "{}{} ", indent, settings.block_name_prefix).unwrap();
            if block.hidden {
                write!(write, "{}", settings.hidden_prefix).unwrap();
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
                print_line(&line, settings, true, indent, newline, write);
            }
        }
        write!(write, "{}{}", fence_sequence, newline).unwrap();
    }

    /// Prints a line of a code block
    fn print_line(
        line: &Line,
        settings: &ParserSettings,
        print_comments: bool,
        indent: &str,
        newline: &str,
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
        write!(write, "{}", newline).unwrap();
    }

    #[cfg(test)]
    mod tests {
        use crate::config::Config;
        use crate::document::{CodeBlock, Line, Source};

        #[test]
        fn print_code_block() {
            let config = toml::from_str::<Config>(include_str!("create/Yarner.toml")).unwrap();

            let code = CodeBlock {
                line_number: 1,
                indent: "".to_string(),
                name: Some("Code block".to_string()),
                is_unnamed: false,
                language: Some("rust".to_string()),
                hidden: false,
                alternative: false,
                source_file: None,
                source: vec![
                    Line {
                        indent: "    ".to_string(),
                        source: Source::Source("fn main() {}".to_string()),
                        comment: None,
                    },
                    Line {
                        indent: "    ".to_string(),
                        source: Source::Macro("Another block".to_string()),
                        comment: None,
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
    use crate::document::{CodeBlock, Line, Source};
    use crate::util::TryCollectExt;
    use std::collections::HashMap;
    use std::fmt::Write;

    /// Formats this `Document` as a string containing the compiled code
    pub fn print_code(
        code_blocks: &HashMap<Option<&str>, Vec<&CodeBlock>>,
        entry_blocks: &[&CodeBlock],
        settings: Option<&LanguageSettings>,
        newline: &str,
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
            write!(
                result,
                "{}{}",
                compile_code_block(block, &code_blocks, settings, newline)?,
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
    ) -> Result<String, CompileError> {
        let line_offset = block.line_number;
        block
            .source
            .iter()
            .enumerate()
            .map(|(idx, line)| {
                compile_line(&line, line_offset + idx, code_blocks, settings, newline)
            })
            .try_collect()
            .map(|lines| lines.join(newline))
            .map_err(CompileError::Multi)
    }

    fn compile_line(
        line: &Line,
        line_number: usize,
        code_blocks: &HashMap<Option<&str>, Vec<&CodeBlock>>,
        settings: Option<&LanguageSettings>,
        newline: &str,
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
        match &line.source {
            Source::Source(string) => {
                if blank_lines && string.trim().is_empty() {
                    Ok("".to_string())
                } else {
                    Ok(format!("{}{}", line.indent, string))
                }
            }
            Source::Macro(name) => {
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
                            &line.indent,
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

                    let code = compile_code_block(block, code_blocks, settings, newline)?;
                    for ln in code.lines() {
                        if blank_lines && ln.trim().is_empty() {
                            write!(result, "{}", newline).unwrap();
                        } else {
                            write!(result, "{}{}{}", line.indent, ln, newline).unwrap();
                        }
                    }

                    if !clean && idx == blocks.len() - 1 {
                        write!(
                            result,
                            "{}{} {}{}{}{}{}{}{}{}",
                            &line.indent,
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
                    for error in errors {
                        writeln!(f, "{}", error)?;
                    }
                    Ok(())
                }
                CompileError::Single { line_number, kind } => {
                    writeln!(f, "{:?} (line {})", kind, line_number)
                }
            }
        }
    }

    impl std::error::Error for CompileError {}
}
