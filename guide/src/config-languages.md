# Languages

Sections `language.<lang>` of a project's `Yarner.toml` contain language-specific settings mainly used for the [Reverse mode](./reverse-mode.md).

[[_TOC_]]

## Overview

Language settings are optional. However, they are required for a all languages/file extensions to be used in reverse mode.

Language settings are a section per language, identified from file extensions. Each section looks like this example for Rust (`.rs` files):

```toml
[language.rs]
clear_blank_lines = true
eof_newline = true

    [language.rs.block_labels]
    comment_start = "//"
    # comment_end = "*/"
    block_start = "<@"
    block_next = "<@>"
    block_end = "@>"
```

When language settings are requires to set `clear_blank_lines` or `eof_newline`,
but block labels in the target language are not wanted or not supported, leave out section `[block_labels]`.

## Options

| Option              | Details                                                                                                                             |
| ------------------- | ----------------------------------------------------------------------------------------------------------------------------------- |
| `clear_blank_lines` | Replaces lines containing only whitespaces by blank lines, in code output. Defaults to `true` when no language settings are present |
| `eof_newline`       | Enforces code files to always end with a blank line. Defaults to `true` when no language settings are present                       |
| `comment_start`     | Start of comments in the language. Used for code block labels for reverse mode. Can be start of line or block comments              |
| `comment_end`       | End of comments. Optional, only for languages that support only block comments                                                      |
| `block_start`       | Start sequence of block labels                                                                                                      |
| `block_next`        | Start of next block with the same name                                                                                              |
| `block_end`         | End of block labels                                                                                                                 |
