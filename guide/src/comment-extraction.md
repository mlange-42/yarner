# Comment extraction

[[_TOC_]]

## Comment extraction

By default, the comment extraction sequence is set to `//-` for familiarity, but also to distinguish it from the sequence `//` used by many languages for comments, while playing nicely with Markdown syntax highlighting.

Text following the comment sequence identifies the code block name when used in the first line of a block (see chapter [Blocks and macros](./blocks-and-macros.md)).

Any text after (and including) this sequence is extracted from the code block, and not rendered to the code output.
Note that, since the comments are removed completely when compiling, they do not have to use the actual line comment indicator from you programming language.
In fact, it may be better to choose a sequence that is *not* the regular comment indicator so that you can still have comments in your code output (as with the default `//-`).

## Aside tags

Now that these comments are extracted, it is possible to handle them differently in the documentation output.
In particular, extracted comments can be put in `<aside>` tags, which can then be rendered nicely using CSS.
This is enabled by setting `comments_as_aside = true` in section `parser` of a project's `Yarner.toml` (see [Parser configuration](./config-parser.md)).

## Limitations

Comment extraction does not work with the reverse mode.
As Yarner's special comments (`//-` by default) are not written to code files, they will be lost during reverse mode.

This may be fixed in a future version of Yarner.
