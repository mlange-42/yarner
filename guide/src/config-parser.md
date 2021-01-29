# Parser

Section `parser` of a project's `Yarner.toml` contains all configuration options for Yarner's syntax within Markdown sources.

[[_TOC_]]

## Overview

The default `parser` section as generated by `yarner create <file>` looks like this (comments removed):

```toml
[parser]

fence_sequence = "```"
fence_sequence_alt = "~~~"

comment_start = "//-"
comments_as_aside = false

interpolation_start = "@{"
interpolation_end = "}"

macro_start = "// ==>"
macro_end = "."

transclusion_start = "@{{"
transclusion_end = "}}"
link_prefix = "@"

variable_sep = ":"

file_prefix = "file:"
hidden_prefix = "hidden:"
# default_language = "rust"
```

## Options

| Option                                    | Details                                                                                                                                                                |
| ----------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `fence_sequence`                          | The sequence used for normal code blocks                                                                                                                               |
| `fence_sequence_alt`                      | Alternative fence sequence to allow for code blocks inside code blocks. Use this for the outer block                                                                   |
| `comment_start`                           | Sequence for comments. The comment in the first line of a code block is the block's name                                                                               |
| `comments_as_aside`                       | Comments starting with the above sequence are excluded from code output. With this option on `true`, comments are places in `<aside>` tags in the documentation output |
| `interpolation_start` `interpolation_end` | Start and end of a meta variable                                                                                                                                       |
| `macro_start` `macro_end`                 | Start and end of a macro invocation                                                                                                                                    |
| `transclusion_start` `transclusion_end`   | Start and end of a transclusion. E.g. `@{{transclude.md}}`                                                                                                             |
| `link_prefix`                             | Prefix for links to make Yarner include the linked file in the build process. E.g. `@[Linked file](linked.md)`                                                         |
| `variable_sep`                            | Separator between variable name and value                                                                                                                              |
| `file_prefix`                             | Prefix to treat block names as target file specifiers. E.g. `//- file:main.rs`                                                                                         |
| `hidden_prefix`                           | Prefix to hide a code block in documentation output. E.g. `//- hidden:Secret code block`                                                                               |
| `default_language`                        | The default language in case no language is given after the opening fence sequence. Optional                                                                           |