use crate::config::{ParserSettings, LINK_REGEX};
use crate::document::{CodeBlock, Document, Line, Node, Source, TextBlock, Transclusion};
use crate::util::Fallible;
use std::error::Error;
use std::fmt::Write;
use std::path::{Path, PathBuf};

#[allow(clippy::nonminimal_bool)]
pub fn parse(
    input: &str,
    root_file: &Path,
    path: &Path,
    is_reverse: bool,
    settings: &ParserSettings,
) -> Fallible<(Document, Vec<PathBuf>)> {
    let mut nodes: Vec<Node> = vec![];
    let mut errors: Vec<Box<dyn Error>> = vec![];
    let mut links: Vec<PathBuf> = vec![];

    for (line_number, line) in input.lines().enumerate() {
        let (is_code, is_alt_fenced_code) = if let Some(Node::Code(code_block)) = nodes.last() {
            (true, code_block.alternative)
        } else {
            (false, false)
        };
        let starts_fenced_alt = line.trim_start().starts_with(&settings.fence_sequence_alt);
        let starts_fenced = if starts_fenced_alt {
            false
        } else {
            line.trim_start().starts_with(&settings.fence_sequence)
        };

        if (!is_code && (starts_fenced || starts_fenced_alt))
            || (is_code && starts_fenced && !is_alt_fenced_code)
            || (is_code && starts_fenced_alt && is_alt_fenced_code)
        {
            let fence_sequence = if starts_fenced_alt {
                &settings.fence_sequence_alt
            } else {
                &settings.fence_sequence
            };
            match nodes.last_mut() {
                Some(Node::Code(code_block)) => {
                    if !line.starts_with(&code_block.indent) {
                        errors
                            .push(format!("Incorrect indentation in line {}", line_number).into());
                    }
                    nodes.push(Node::Text(TextBlock::new()));
                }
                _previous => {
                    let code_block = start_code(
                        line,
                        fence_sequence,
                        &settings.default_language,
                        starts_fenced_alt,
                    );
                    nodes.push(Node::Code(code_block));
                }
            }
        } else {
            match nodes.last_mut() {
                Some(Node::Code(block)) => {
                    if line.starts_with(&block.indent) {
                        let error = extend_code(line, line_number, settings, block);

                        if let Some(err) = error {
                            errors.push(err);
                        }
                    } else {
                        errors.push(format!("Incorrect indentation line {}", line_number).into());
                    }
                }

                other => {
                    let block = if let Some(Node::Text(block)) = other {
                        Some(block)
                    } else {
                        None
                    };
                    let (node, error) = start_or_extend_text(
                        &line,
                        line_number,
                        root_file,
                        path,
                        settings,
                        is_reverse,
                        &mut links,
                        block,
                    );
                    if let Some(node) = node {
                        nodes.push(node);
                    }
                    if let Some(error) = error {
                        errors.push(error);
                    }
                }
            }
        }
    }

    if let Some(Node::Text(text)) = nodes.last() {
        if text.lines().is_empty() {
            nodes.pop();
        }
    }

    if !errors.is_empty() {
        let mut msg = String::new();
        for error in errors {
            writeln!(&mut msg, "{}", error).unwrap();
        }
        return Err(msg.into());
    }

    Ok((Document::new(nodes), links))
}

fn start_code(
    line: &str,
    fence_sequence: &str,
    default_language: &Option<String>,
    is_alt_fenced: bool,
) -> CodeBlock {
    let indent_len = line.find(fence_sequence).unwrap();
    let (indent, rest) = line.split_at(indent_len);
    let rest = &rest[fence_sequence.len()..];

    let mut code_block = CodeBlock::new().indented(indent);

    let language = rest.trim();
    let language = if language.is_empty() {
        match default_language {
            Some(language) => Some(language.to_owned()),
            None => None,
        }
    } else {
        Some(language.to_owned())
    };
    if let Some(language) = language {
        code_block = code_block.in_language(language);
    }
    code_block.alternative(is_alt_fenced)
}

fn extend_code(
    line: &str,
    line_number: usize,
    settings: &ParserSettings,
    block: &mut CodeBlock,
) -> Option<Box<dyn Error>> {
    let mut error = None;
    if block.source.is_empty() && line.trim().starts_with(&settings.block_name_prefix) {
        let name = line.trim()[settings.block_name_prefix.len()..].trim();

        if let Some(stripped) = name.strip_prefix(&settings.hidden_prefix) {
            block.name = Some(stripped.to_string());
            block.hidden = true;
        } else {
            block.name = Some(name.to_string());
        };
    } else {
        let line = match parse_line(line_number, &line[block.indent.len()..], settings) {
            Ok(line) => Some(line),
            Err(err) => {
                error = Some(format!("{} (line {})", err, line_number).into());
                None
            }
        };
        if let Some(line) = line {
            block.add_line(line);
        }
    }
    error
}

#[allow(clippy::too_many_arguments)]
fn start_or_extend_text(
    line: &str,
    line_number: usize,
    root_file: &Path,
    path: &Path,
    settings: &ParserSettings,
    is_reverse: bool,
    mut links: &mut Vec<PathBuf>,
    block: Option<&mut TextBlock>,
) -> (Option<Node>, Option<Box<dyn Error>>) {
    let parsed = parse_links(&line, root_file, path, settings, !is_reverse, &mut links);
    let line = if parsed.is_some() {
        parsed.as_ref().unwrap()
    } else {
        line
    };
    let mut node = None;
    let mut error = None;
    match parse_transclusion(line, path, settings) {
        Err(err) => error = Some(format!("{} (line {})", err, line_number).into()),
        Ok(trans) => match trans {
            Some(nd) => {
                node = Some(nd);
            }
            None => {
                if let Some(block) = block {
                    block.add_line(line);
                } else {
                    let mut new_block = TextBlock::new();
                    new_block.add_line(line);
                    node = Some(Node::Text(new_block));
                };
            }
        },
    }

    (node, error)
}

fn parse_transclusion(
    line: &str,
    into: &Path,
    settings: &ParserSettings,
) -> Fallible<Option<Node>> {
    if let Some(rest) = line.trim().strip_prefix(&settings.transclusion_start) {
        if let Some(index) = rest.find(&settings.transclusion_end) {
            let trans = &rest[..index];

            let target = LINK_REGEX
                .captures_iter(trans)
                .map(|match_| match_.get(2).unwrap().as_str())
                .next()
                .unwrap_or(&trans);

            let path = into.parent().unwrap_or_else(|| Path::new(".")).join(target);

            Ok(Some(Node::Transclusion(Transclusion::new(
                PathBuf::from(path_clean::clean(
                    &path.to_str().unwrap().replace("\\", "/"),
                )),
                line.to_owned(),
            ))))
        } else {
            Err(format!("Unclosed transclusion in: {}", line).into())
        }
    } else {
        Ok(None)
    }
}

/// Parses a line as code, returning the parsed `Line` object
fn parse_line(line_number: usize, input: &str, settings: &ParserSettings) -> Fallible<Line> {
    let indent_len = input.chars().take_while(|ch| ch.is_whitespace()).count();
    let (indent, rest) = input.split_at(indent_len);

    // TODO: Temporarily disables comment extraction.
    let (rest, comment) = (rest, None);
    /*let (rest, comment) = if let Some(comment_index) = rest.find(&settings.block_name_prefix) {
        let (rest, comment) = rest.split_at(comment_index);
        (
            rest,
            Some((&comment[settings.block_name_prefix.len()..]).to_owned()),
        )
    } else {
        (rest, None)
    };*/

    if let Some(stripped) = rest.strip_prefix(&settings.macro_start) {
        if let Some(name) = stripped.strip_suffix(&settings.macro_end) {
            return Ok(Line {
                line_number,
                indent: indent.to_owned(),
                source: Source::Macro(name.trim().to_owned()),
                comment,
            });
        }
    }

    Ok(Line {
        line_number,
        indent: indent.to_owned(),
        source: Source::Source(rest.to_owned()),
        comment,
    })
}

fn parse_links(
    line: &str,
    _root_file: &Path,
    from: &Path,
    settings: &ParserSettings,
    remove_marker: bool,
    links_out: &mut Vec<PathBuf>,
) -> Option<String> {
    let marker = &settings.link_prefix;
    let regex = &LINK_REGEX;

    let mut offset = 0;
    let mut new_line: Option<String> = None;
    for capture in regex.captures_iter(line) {
        let index = capture.get(0).unwrap().start();
        if line[..index].ends_with(marker) {
            if remove_marker {
                let len = marker.len();
                if let Some(l) = &mut new_line {
                    *l = format!("{}{}", &l[..(index - offset)], &l[(index + len - offset)..]);
                } else {
                    new_line = Some(format!(
                        "{}{}",
                        &line[..(index - offset)],
                        &line[(index + len - offset)..]
                    ));
                }
                offset += len;
            }

            let link = capture.get(2).unwrap().as_str();
            let mut path = from.parent().unwrap().to_path_buf();
            path.push(link);
            if path.is_relative() && is_relative_link(link) {
                let path = PathBuf::from(path_clean::clean(
                    &path.to_str().unwrap().replace("\\", "/"),
                ));
                links_out.push(path);
            }
        }
    }
    new_line
}

fn is_relative_link(link: &str) -> bool {
    !link.starts_with('#')
        && !link.starts_with("http://")
        && !link.starts_with("https://")
        && !link.starts_with("ftp://")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::Source::Macro;

    #[test]
    fn parse_text() {
        let settings = default_settings();
        let text = r#"# Caption

text
"#;
        let (doc, links) = parse(
            text,
            Path::new("README.md"),
            Path::new("README.md"),
            false,
            &settings,
        )
        .unwrap();

        assert_eq!(doc.nodes.len(), 1);
        assert_eq!(links.len(), 0);
        matches!(doc.nodes[0], Node::Text(_));
    }

    #[test]
    fn parse_code() {
        let settings = default_settings();
        let text = r#"# Caption

```
//- Code
code
// ==> Macro.
```

text
"#;
        let (doc, links) = parse(
            text,
            Path::new("README.md"),
            Path::new("README.md"),
            false,
            &settings,
        )
        .unwrap();

        assert_eq!(doc.nodes.len(), 3);
        assert_eq!(links.len(), 0);
        assert!(if let Node::Code(block) = &doc.nodes[1] {
            assert_eq!(links.len(), 0);
            assert_eq!(block.name, Some(String::from("Code")));
            assert_eq!(block.source.len(), 2);
            if let Macro(name) = &block.source[1].source {
                assert_eq!(name, "Macro");
                true
            } else {
                false
            }
        } else {
            false
        });
    }

    #[test]
    fn parse_transclusion() {
        let settings = default_settings();
        let text = r#"# Caption

@{{[test.md](test.md)}}

text
"#;
        let (doc, links) = parse(
            text,
            Path::new("README.md"),
            Path::new("README.md"),
            false,
            &settings,
        )
        .unwrap();

        assert_eq!(doc.nodes.len(), 3);
        assert_eq!(links.len(), 0);
        assert!(if let Node::Transclusion(trans) = &doc.nodes[1] {
            trans.file() == &PathBuf::from("test.md")
        } else {
            false
        });
    }

    #[test]
    fn parse_link() {
        let settings = default_settings();
        let text = r#"# Caption

@[test.md](test.md)

text
"#;
        let (doc, links) = parse(
            text,
            Path::new("README.md"),
            Path::new("README.md"),
            false,
            &settings,
        )
        .unwrap();

        assert_eq!(doc.nodes.len(), 1);
        assert_eq!(links.len(), 1);
        assert_eq!(links[0], PathBuf::from("test.md"));
    }

    fn default_settings() -> ParserSettings {
        ParserSettings {
            fence_sequence: "```".to_string(),
            fence_sequence_alt: "~~~".to_string(),
            default_language: None,
            comments_as_aside: false,
            block_name_prefix: "//-".to_string(),
            macro_start: "// ==>".to_string(),
            macro_end: ".".to_string(),
            transclusion_start: "@{{".to_string(),
            transclusion_end: "}}".to_string(),
            link_prefix: "@".to_string(),
            file_prefix: "file:".to_string(),
            hidden_prefix: "hidden:".to_string(),
        }
    }
}
