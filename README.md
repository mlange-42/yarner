# Outline

[![](https://img.shields.io/crates/v/outline.svg)](https://crates.io/crates/outline)
![](https://img.shields.io/crates/l/outline.svg)
[![](https://docs.rs/outline/badge.svg)](https:///docs.rs/outline)

Generic literate programming transpiler. This project aims to provide a modern, developer friendly
literate programming tool. After a brief search online, other options claiming to be "modern" are
years old, and have some pretty ugly syntax (in my opinion). If I am wrong in this regard, please
open an issue to correct me!

In contrast, Outline works with familiar syntax, which can be further customized to suit your needs
exactly. It uses pluggable, configurable input formats, with out-of-the-box support for:
*   Markdown;
*   Latex;
*   HTML; and
*   (some approximation of) Bird style

See the examples directory for full working examples in each style.

## Installation

The provided binary (honestly a quick hack to get things working) can be installed via cargo:

```bash
cargo install outline --features bin
```

The `outline` crate is also available to allow you to write your own parser:

```toml
outline = "0.1.0"
```

If you write your own parser, feel free to continue hacking on the provided `src/bin/main.rs` to
support your new parser.

## Features

In all styles, the code sections are handled the same way, supporting:
*   macros;
*   meta variable interpolation;
*   comment extraction;
*   named entrypoints; and
*   multiple languages in one file

The text sections are also handled the same way in all styles - just copied in and written out with
no processing. This allows you to write your documentation however you like. Currently the only
difference between the parsers is the way they detect the start and end of a code block. Because of
this the weaved documentation file will look very similar to the original literate source, with only
slight changes to the code block syntax to ensure that they are valid in the true documentation
language. Given this, note that any post-processing or format changes you wish to apply to the
documentation should be performed on the generated document.

### Macros

Macros are what enables the literate program to be written in logical order for the human reader.
Using Outline, this is accomplished by naming the code blocks, and then referencing them later by
"invoking" the macro. While the syntax for naming a code block is specific to the documentation
style, macro invocation is the same.

By default, macro invocations start with a long arrow `==>` and end with a period `.`. Both of these
sequences can be customized to suit your needs better. The only restriction with macro invocations
is that they must be the only thing on the line. That is, this is valid:

```rs
fn main() {
  ==> Calculate the very complex result.
  ==> Print the results for the user.
}
```

But this would not invoke the macro named within the `if()` as the macro sequences do not start and
end the line:

```rs
fn main() {
  if (==> A very complex condition is true.) {
    ==> Do something cool.
  }
}
```

Another feature of macros to note is that if two code blocks have the same name, they are
concatenated, in the order they are written. This can be very useful in defining global variables or
listing imports closer to the parts where they are used.

### Meta Variables

If you consider a macro invocation like a function call, then meta variables are like parameters.

By default, to indicate that a macro includes a meta variable, the name of the variable must be part
of the name of the macro, delimited by `@{` and `}`.

Then that meta variable may be used within the macro by again using its name within the `@{` and `}`
in the code.

Finally, a macro with meta variables is invoked by replacing the name of the variable with its
value in the invocation.

An example:

```tex
Here is our macro with meta variables:

\begin{code}[language=rs,name={Say @{something} to @{someone}}]
println!("Hey, @{someone}! I was told to tell you \"@{something}\"");
\end{code}

Now, to say things to many people:

\begin{code}[language=rs]
==> Say @{Hello} to @{Jim}.
==> Say @{How are you} to @{Tom}.
==> Say @{I am good!} to @{Angela}.
\end{code}
```

This feature allows for more flexibility when writing macros, as well as possibly making the intent
clearer.

### Extracted comments

By default, the comment extraction sequence is set to `//`, purely for familiarity. Any text after
(and including) this sequence is extracted from the code block, and not rendered to the tangled
source code. Note that, since the comments are removed completely when compiling, they do not have
to use the actual line comment indicator from you programming language. In fact, it may be better to
choose a sequence that is *not* the regular comment indicator so that you can still have comments in
your tangled code output.

Now that these comments are extracted, it is possible to handle them differently in the weaved
documentation file. Though some formats do not support any special behaviour, and simply write these
comments back into the code, some are able to provide special rendering. In particular, the standard
Markdown and HTML styles are able to render extracted comments in `<aside>` tags, which can then be
rendered nicely using CSS.

See the HTML example for an example of one way to render the extracted comments.

### Named Entrypoints

By default, the entrypoint of the program is always the unnamed code block. However, this limits the
output of one input file to always be the same source code. It also means that you can't have a name
on the entrypoint in the documentation, which could be useful.

To get around this, an entrypoint name can be passed to Outline on the command line. Then, instead
of starting at the unnamed code block, it will start at the code block with this name.

Note that if you use a named entrypoint, there is no way to reference the unnamed code blocks as
macros. You can, however, use the unnamed code blocks to provide examples, for example, to the
readers of the documentation, so they are still useful.

### Multiple languages

Some documentation formats allow you to indicate the language that a code block is written in. In
fact, it is recommended that you always include the language when you write a code block,
particularly if multiple programming languages are used within the same document.

By properly labelling all code blocks, it is then possible to write a program in multiple
programming languages at once. Whether this is practical or not remains to be seen, but it is
supported nonetheless. By then supplying a language name on the command line, only code blocks in
that language are used when generating the tangled source. For example, here is a trivial program
written in two languages:

```tex
Here we have hello world in Ruby:

\begin{code}[language=rb]
puts "Hello world"
\end{code}

And here it is again in Rust:

\begin{code}[language=rs]
fn main() {
  println!("Hello world");
}
\end{code}
```

Compiling this with no language supplied with just ignore language information, so a single output
will be generated containing both languages. However, supplying the `--language rb` flag to Outline
will cause only the code blocks tagged with `rb` will be used to generate code.

## Usage

```
Outline 1.0
Cameron Eldridge <cameldridge@gmail.com>
Literate programming compiler

USAGE:
    outline [OPTIONS] [input]...

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -o, --output <code_dir>          Output tangled code files to this directory. No code files will be printed by
                                     default.
    -c, --config <config_file>       Sets the config file name [default: Outline.toml]
    -d, --docs <doc_dir>             Directory to output weaved documentation files to. No documentation will be printed
                                     by default.
    -e, --entrypoint <entrypoint>    The named entrypoint to use when tangling code. Defaults to the unnamed code block.
    -l, --language <language>        The language to output the tangled code in. Only code blocks in this language will
                                     be used.
    -s, --style <style>              Sets the style to use. If not specified, it is inferred from the file extension.
                                     When reading from STDIN, defaults to 'md'. [possible values: bird, md, tex, html]

ARGS:
    <input>..
```

### Configuration

Each style supports some additional configuration, which is provided via a toml configuration file
(default: Outline.toml). Multiple styles can be configured at once in the configuration file. Note
that if a style appears in the configuration file, its full set of options is required (all defaults
will be discarded).

For more information on these options, see the [API documentation](https://docs.rs/outline).

```toml
[tex]
code_environment = "code"
default_language = "rs" # optional
comment_start = "//"
interpolation_start = "@{"
interpolation_end = "}"
macro_start = "==> "
macro_end = "."

[md]
fence_sequence = "```"
block_name_start = " - "
comments_as_aside = false
default_language = "rs" # optional
comment_start = "//"
interpolation_start = "@{"
interpolation_end = "}"
macro_start = "==> "
macro_end = "."

[html]
code_tag = "code"
language_attribute = "data-language"
name_attribute = "data-name"
block_class = "block"
language_class = "language-{}"
comments_as_aside = true
default_language = "rs" # optional
comment_start = "//"
interpolation_start = "@{"
interpolation_end = "}"
macro_start = "==> "
macro_end = "."

[bird]
code_marker = "> "
code_name_marker = ">>> "
comment_start = "//"
interpolation_start = "@{"
interpolation_end = "}"
macro_start = "==> "
macro_end = "."
```

### Extending

It is possible to write your own Outline parsers for more formats, or to extend the existing
formats.

To do this, you need to implement three traits - `Parser`, `Printer`, and `ParserConfig`.

The `Parser` trait is responsible for deciding where a code block starts and ends, and creating
a `Document` based on that.

The `Printer` is responsible for taking parsed code blocks and writing them back out, potentially in
a more "valid" form than they were parsed in.

The `ParserConfig` exposes the most common configuration options in order to implement the core
functionality, such as macro invocation, meta variables, and comment extraction.

For now, if you wish to write your own parser, I recommend looking to the existing parsers as your
starting point, and then looking to `src/bin/main.rs` for an example of how to use your completed
parser.

Additionally, the API documentation is another good place to look.
