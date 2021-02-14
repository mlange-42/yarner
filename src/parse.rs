use crate::config::{ParserSettings, LINK_REGEX};
use crate::document::{CodeBlock, Document, Line, Node, Source, TextBlock, Transclusion};
use crate::util::Fallible;
use regex::Captures;
use std::error::Error;
use std::fmt::Write;
use std::ops::Deref;
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
                    let code_block = start_code(line, fence_sequence, starts_fenced_alt);
                    nodes.push(Node::Code(code_block));
                }
            }
        } else {
            match nodes.last_mut() {
                Some(Node::Code(block)) => {
                    if line.starts_with(&block.indent) {
                        extend_code(line, line_number, settings, block);
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

fn start_code(line: &str, fence_sequence: &str, is_alt_fenced: bool) -> CodeBlock {
    let indent_len = line.find(fence_sequence).unwrap();
    let (indent, rest) = line.split_at(indent_len);
    let rest = &rest[fence_sequence.len()..];

    let mut code_block = CodeBlock::new().indented(indent);

    let language = rest.trim();
    let language = if language.is_empty() {
        None
    } else {
        Some(language.to_owned())
    };
    if let Some(language) = language {
        code_block = code_block.in_language(language);
    }
    code_block.alternative(is_alt_fenced)
}

fn extend_code(line: &str, line_number: usize, settings: &ParserSettings, block: &mut CodeBlock) {
    if block.source.is_empty() && line.trim().starts_with(&settings.block_name_prefix) {
        let name = line.trim()[settings.block_name_prefix.len()..].trim();

        if let Some(stripped) = name.strip_prefix(&settings.hidden_prefix) {
            block.name = Some(stripped.to_string());
            block.hidden = true;
        } else {
            block.name = Some(name.to_string());
        };
    } else {
        let line = parse_line(line_number, &line[block.indent.len()..], settings);
        block.add_line(line);
    }
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
    let parsed = parse_links(&line, root_file, path, settings, is_reverse, &mut links);
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
        if let Some(trans) = rest.strip_suffix(&settings.transclusion_end) {
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
fn parse_line(line_number: usize, input: &str, settings: &ParserSettings) -> Line {
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
            return Line {
                line_number,
                indent: indent.to_owned(),
                source: Source::Macro(name.trim().to_owned()),
                comment,
            };
        }
    }

    Line {
        line_number,
        indent: indent.to_owned(),
        source: Source::Source(rest.to_owned()),
        comment,
    }
}

fn parse_links(
    line: &str,
    root_file: &Path,
    from: &Path,
    settings: &ParserSettings,
    is_reverse: bool,
    links_out: &mut Vec<PathBuf>,
) -> Option<String> {
    let marker = &settings.link_following_pattern.0;
    let regex = &settings.link_following_pattern.1;

    if regex.is_match(line) {
        if is_reverse {
            for caps in regex.captures_iter(line) {
                let follow = caps.get(1).is_some();
                if let Some(path) = absolute_link(&caps[3], from) {
                    if follow {
                        links_out.push(path);
                    }
                }
            }
            None
        } else {
            let ln = regex
                .replace_all(line, |caps: &Captures| {
                    let label = &caps[2];
                    let link = &caps[3];
                    let follow = caps.get(1).is_some();

                    if let Some(path) = absolute_link(link, from) {
                        let new_link = relative_link(&path, root_file);

                        let line = format!(
                            "{}[{}]({})",
                            if is_reverse && follow { marker } else { "" },
                            if label == link { &new_link } else { label },
                            new_link,
                        );

                        if follow {
                            links_out.push(path);
                        }
                        line
                    } else {
                        format!(
                            "{}[{}]({})",
                            if is_reverse && follow { marker } else { "" },
                            label,
                            link,
                        )
                    }
                })
                .deref()
                .to_owned();
            Some(ln)
        }
    } else {
        None
    }
}

fn absolute_link<P>(link: &str, from: P) -> Option<PathBuf>
where
    P: AsRef<Path>,
{
    if is_relative_link(&link) {
        let mut path = from.as_ref().parent().unwrap().to_path_buf();
        path.push(link);

        let path = PathBuf::from(path_clean::clean(
            &path.to_str().unwrap().replace("\\", "/"),
        ));

        Some(path)
    } else {
        None
    }
}

fn relative_link<P, B>(abs_link: P, root: B) -> String
where
    P: AsRef<Path>,
    B: AsRef<Path>,
{
    pathdiff::diff_paths(&abs_link, root.as_ref().parent().unwrap())
        .and_then(|p| p.as_path().to_str().map(|s| s.replace('\\', "/")))
        .unwrap_or_else(|| "invalid path".to_owned())
}

fn is_relative_link(link: &str) -> bool {
    !link.starts_with('#')
        && !link.contains("file://")
        && !link.contains("http://")
        && !link.contains("https://")
        && !link.contains("ftp://")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::LINK_PATTERN;
    use crate::document::Source::Macro;
    use regex::Regex;

    #[test]
    fn absolute_link() {
        assert_eq!(
            super::absolute_link("linked.md", "README.md"),
            Some(PathBuf::from("linked.md"))
        );

        assert_eq!(
            super::absolute_link("linked.md", "src/README.md"),
            Some(PathBuf::from("src/linked.md"))
        );

        assert_eq!(
            super::absolute_link("../linked.md", "src/README.md"),
            Some(PathBuf::from("linked.md"))
        );
    }

    #[test]
    fn relative_link() {
        assert_eq!(super::relative_link("linked.md", "README.md"), "linked.md");
        assert_eq!(
            super::relative_link("src/linked.md", "README.md"),
            "src/linked.md"
        );
        assert_eq!(
            super::relative_link("src/linked.md", "src/README.md"),
            "linked.md"
        );
        assert_eq!(
            super::relative_link("docs/linked.md", "src/README.md"),
            "../docs/linked.md"
        );
        assert_eq!(
            super::relative_link("linked.md", "src/README.md"),
            "../linked.md"
        );
    }

    #[test]
    fn parse_single_link() {
        let settings = default_settings();
        let from = Path::new("README.md");
        let root = Path::new("README.md");

        let line = "A single @[link](link-1.md).";
        let mut links = vec![];
        let new_line = super::parse_links(line, &root, &from, &settings, false, &mut links);
        assert_eq!(new_line, Some("A single [link](link-1.md).".to_owned()));
        assert_eq!(links.len(), 1);
        assert_eq!(links[0], PathBuf::from("link-1.md"));
    }

    #[test]
    fn parse_two_links() {
        let settings = default_settings();
        let from = Path::new("README.md");
        let root = Path::new("README.md");

        let line = "One @[link](link-1.md) and another @[link](link-2.md).";
        let mut links = vec![];
        let new_line = super::parse_links(line, &root, &from, &settings, false, &mut links);
        assert_eq!(
            new_line,
            Some("One [link](link-1.md) and another [link](link-2.md).".to_owned())
        );
        assert_eq!(links.len(), 2);
        assert_eq!(links[0], PathBuf::from("link-1.md"));
        assert_eq!(links[1], PathBuf::from("link-2.md"));
    }

    #[test]
    fn parse_parent_folder_link() {
        let settings = default_settings();
        let from = Path::new("src/README.md");
        let root = Path::new("src/README.md");

        let line = "A single @[link](../link-1.md).";
        let mut links = vec![];
        let new_line = super::parse_links(line, &root, &from, &settings, false, &mut links);
        assert_eq!(new_line, Some("A single [link](../link-1.md).".to_owned()));
        assert_eq!(links.len(), 1);
        assert_eq!(links[0], PathBuf::from("link-1.md"));
    }

    #[test]
    fn parse_sibling_folder_link() {
        let settings = default_settings();
        let from = Path::new("src/README.md");
        let root = Path::new("src/README.md");

        let line = "A single @[link](../docs/link-1.md).";
        let mut links = vec![];
        let new_line = super::parse_links(line, &root, &from, &settings, false, &mut links);
        assert_eq!(
            new_line,
            Some("A single [link](../docs/link-1.md).".to_owned())
        );
        assert_eq!(links.len(), 1);
        assert_eq!(links[0], PathBuf::from("docs/link-1.md"));
    }

    #[test]
    fn parse_transcluded_child_folder_link() {
        let settings = default_settings();
        let from = Path::new("src/transcluded.md");
        let root = Path::new("README.md");

        let line = "A single @[link](link-1.md).";
        let mut links = vec![];
        let new_line = super::parse_links(line, &root, &from, &settings, false, &mut links);
        assert_eq!(new_line, Some("A single [link](src/link-1.md).".to_owned()));
        assert_eq!(links.len(), 1);
        assert_eq!(links[0], PathBuf::from("src/link-1.md"));
    }

    #[test]
    fn parse_transcluded_sibling_folder_link() {
        let settings = default_settings();
        let from = Path::new("docs/transcluded.md");
        let root = Path::new("src/README.md");

        let line = "A single @[link](link-1.md).";
        let mut links = vec![];
        let new_line = super::parse_links(line, &root, &from, &settings, false, &mut links);
        assert_eq!(
            new_line,
            Some("A single [link](../docs/link-1.md).".to_owned())
        );
        assert_eq!(links.len(), 1);
        assert_eq!(links[0], PathBuf::from("docs/link-1.md"));
    }

    #[test]
    fn parse_transcluded_parent_folder_link() {
        let settings = default_settings();
        let from = Path::new("transcluded.md");
        let root = Path::new("src/README.md");

        let line = "A single @[link](docs/link-1.md).";
        let mut links = vec![];
        let new_line = super::parse_links(line, &root, &from, &settings, false, &mut links);
        assert_eq!(
            new_line,
            Some("A single [link](../docs/link-1.md).".to_owned())
        );
        assert_eq!(links.len(), 1);
        assert_eq!(links[0], PathBuf::from("docs/link-1.md"));
    }

    #[test]
    fn parse_absolute_link() {
        let settings = default_settings();
        let from = Path::new("README.md");
        let root = Path::new("README.md");

        let line = "An absolute @[link](https://github.com/mlange-42/yarner).";
        let mut links = vec![];
        let new_line = super::parse_links(line, &root, &from, &settings, false, &mut links);
        assert_eq!(
            new_line,
            Some("An absolute [link](https://github.com/mlange-42/yarner).".to_owned())
        );
        assert_eq!(links.len(), 0);
    }

    #[test]
    fn parse_single_link_reverse() {
        let settings = default_settings();
        let from = Path::new("docs/README.md");
        let root = Path::new("docs/README.md");

        let line = "A single @[link](link-1.md).";
        let mut links = vec![];
        let new_line = super::parse_links(line, &root, &from, &settings, true, &mut links);
        assert_eq!(new_line, None);
        assert_eq!(links.len(), 1);
        assert_eq!(links[0], PathBuf::from("docs/link-1.md"));
    }

    #[test]
    fn parse_doc_text() {
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
    fn parse_doc_code() {
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
    fn parse_doc_transclusion() {
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
    fn parse_doc_link() {
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
            comments_as_aside: false,
            block_name_prefix: "//-".to_string(),
            macro_start: "// ==>".to_string(),
            macro_end: ".".to_string(),
            transclusion_start: "@{{".to_string(),
            transclusion_end: "}}".to_string(),
            link_following_pattern: (
                "@".to_string(),
                Regex::new(&format!("(@)?{}", LINK_PATTERN)).unwrap(),
            ),
            file_prefix: "file:".to_string(),
            hidden_prefix: "hidden:".to_string(),
        }
    }
}
