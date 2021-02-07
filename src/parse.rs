use crate::config::{ParserSettings, LINK_REGEX};
use crate::document::{
    code::{CodeBlock, Line, Source},
    text::TextBlock,
    transclusion::Transclusion,
    Document, Node,
};
use crate::util::Fallible;
use std::error::Error;
use std::fmt::Write;
use std::fs::File;
use std::path::{Path, PathBuf};

#[allow(clippy::nonminimal_bool)]
pub fn parse(input: &str, path: &Path, settings: &ParserSettings) -> Fallible<Document> {
    let mut state: Option<Node> = None;
    let mut nodes: Vec<Node> = vec![];
    let mut errors: Vec<Box<dyn Error>> = vec![];

    for (line_number, line) in input.lines().enumerate() {
        let (is_code, is_alt_fenced_code) = if let Some(Node::Code(code_block)) = &state {
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
            match state.take() {
                Some(Node::Code(code_block)) => {
                    if line.starts_with(&code_block.indent) {
                        nodes.push(Node::Code(code_block));
                    } else {
                        errors
                            .push(format!("Incorrect indentation in line {}", line_number).into());
                    }
                }
                previous => {
                    let indent_len = line.find(fence_sequence).unwrap();
                    let (indent, rest) = line.split_at(indent_len);
                    let rest = &rest[fence_sequence.len()..];

                    let mut code_block = CodeBlock::new().indented(indent);

                    let language = rest.trim();
                    let language = if language.is_empty() {
                        match &settings.default_language {
                            Some(language) => Some(language.to_owned()),
                            None => None,
                        }
                    } else {
                        Some(language.to_owned())
                    };
                    if let Some(language) = language {
                        code_block = code_block.in_language(language);
                    }
                    code_block = code_block.alternative(starts_fenced_alt);
                    state = Some(Node::Code(code_block));
                    if let Some(node) = previous {
                        nodes.push(node);
                    }
                }
            }
        } else {
            match &mut state {
                None => {
                    let mut new_block = TextBlock::new();
                    new_block.add_line(line);
                    state = Some(Node::Text(new_block));
                    match parse_transclusion(line, path, settings) {
                        Err(err) => errors.push(format!("{} (line {})", err, line_number).into()),
                        Ok(trans) => {
                            if let Some(node) = trans {
                                let new_block = TextBlock::new();
                                state = Some(Node::Text(new_block));
                                nodes.push(node);
                            }
                        }
                    }
                }
                Some(Node::Text(block)) => match parse_transclusion(line, path, settings) {
                    Err(err) => errors.push(format!("{} (line {})", err, line_number).into()),
                    Ok(trans) => match trans {
                        Some(node) => {
                            let ret = state.take();
                            state = Some(node);
                            nodes.push(ret.unwrap());
                        }
                        None => block.add_line(line),
                    },
                },
                Some(Node::Code(block)) => {
                    if line.starts_with(&block.indent) {
                        if block.source.is_empty()
                            && line.trim().starts_with(&settings.block_name_prefix)
                        {
                            let name = line.trim()[settings.block_name_prefix.len()..].trim();

                            if let Some(stripped) = name.strip_prefix(&settings.hidden_prefix) {
                                block.name = Some(stripped.to_string());
                                block.hidden = true;
                            } else {
                                block.name = Some(name.to_string());
                            };
                        } else {
                            let line = match parse_line(
                                line_number,
                                &line[block.indent.len()..],
                                settings,
                            ) {
                                Ok(line) => Some(line),
                                Err(error) => {
                                    errors.push(format!("{} (line {})", error, line_number).into());
                                    None
                                }
                            };
                            if let Some(line) = line {
                                block.add_line(line);
                            }
                        }
                    } else {
                        errors.push(format!("Incorrect indentation line {}", line_number).into());
                    }
                }
                Some(Node::Transclusion(trans)) => {
                    let trans = trans.clone();
                    let mut new_block = TextBlock::new();
                    new_block.add_line(line);
                    state = Some(Node::Text(new_block));
                    nodes.push(Node::Transclusion(trans));
                }
            }
        }
    }

    if !errors.is_empty() {
        let mut msg = String::new();
        for error in errors {
            writeln!(&mut msg, "{}", error).unwrap();
        }
        return Err(msg.into());
    }

    if let Some(node) = state.take() {
        nodes.push(node);
    }

    Ok(Document::new(nodes))
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
                path,
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

pub fn find_links(
    input: &mut Document,
    from: &Path,
    settings: &ParserSettings,
    remove_marker: bool,
) -> Fallible<Vec<PathBuf>> {
    let regex = &settings.link_following_pattern;
    let mut paths = vec![];

    for block in input.text_blocks_mut() {
        for line in block.lines_mut().iter_mut() {
            let mut offset = 0;
            let mut new_line: Option<String> = None;
            for capture in regex.1.captures_iter(line) {
                if remove_marker {
                    let index = capture.get(0).unwrap().start();
                    let len = regex.0.len();
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
                let path = PathBuf::from(path_clean::clean(
                    &path.to_str().unwrap().replace("\\", "/"),
                ));
                if path.is_relative() && is_relative_link(link) {
                    if File::open(&path).is_ok() {
                        paths.push(path);
                    } else {
                        eprintln!("WARNING: link target not found for {}", path.display());
                    }
                }
            }
            if let Some(new_line) = new_line {
                *line = new_line;
            }
        }
    }
    Ok(paths)
}

fn is_relative_link(link: &str) -> bool {
    !link.starts_with('#')
        && !link.starts_with("http://")
        && !link.starts_with("https://")
        && !link.starts_with("ftp://")
}
