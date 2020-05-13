# Outline

[![Build Status](https://travis-ci.com/mlange-42/outline.svg?branch=master)](https://travis-ci.com/mlange-42/outline)

Generic literate programming transpiler. This project aims to provide a modern, developer friendly
literate programming tool.

Outline works with familiar syntax, which can be further customized to suit your needs
exactly. It uses pluggable, configurable input formats, with out-of-the-box support for:
*   Markdown
*   Latex
*   HTML
*   (some approximation of) Bird style

See the examples directory for full working examples in each style.

## Installation

* Download the [latest binaries](https://github.com/mlange-42/outline/releases) [TODO: provide binaries].
* Unzip somewhere with write privileges (only required for running examples in place).

## Getting started

To set up a new project, use the `create` subcommand.
To create a Markdown project, run:

```
outline create README.md md
```

Or simply, as Markdown is the default:

```
outline create README.md
```

This creates a file `Outline.toml` with default settings,
and a file `README.md` as starting point for Literate Programming.

To "compile" the project (extract code and create documentation),
simply run:
```
outline
```

## Features

In all styles, the code sections are handled the same way, supporting:
*   macros
*   meta variable interpolation
*   comment extraction
*   named entrypoints
*   multiple languages in one file

The text sections are also handled the same way in all styles - just copied in and written out with
no processing. This allows you to write your documentation however you like. Currently the only
difference between the parsers is the way they detect the start and end of a code block. Because of
this the weaved documentation file will look very similar to the original literate source, with only
slight changes to the code block syntax to ensure that they are valid in the true documentation
language. Given this, note that any post-processing or format changes you wish to apply to the
documentation should be performed on the generated document.

Some advanced features are currently only supported for **Markdown**:
* File transclusions (e.g. "draw" sub-documents into the main document)
* Automatic inclusion of linked files

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

~~~md
Here is our macro with meta variables:

```rust - Say @{something} to @{someone}
println!("Hey, @{someone}! I was told to tell you \"@{something}\"");
```

Now, to say things to many people:

```rust
==> Say @{Hello} to @{Jim}.
==> Say @{How are you} to @{Tom}.
==> Say @{I am good!} to @{Angela}.
```
~~~

Meta variables can have default values:

~~~md
Here is our macro with default meta variables:

```rust - Say @{something:Hello} to @{someone}
println!("Hey, @{someone}! I was told to tell you \"@{something}\"");
```

Now, to say the default "Hello" to many people:

```rust
==> Say @{} to @{Jim}.
==> Say @{} to @{Tom}.
==> Say @{} to @{Angela}.
```
~~~

These features allows for more flexibility when writing macros, as well as possibly making the intent
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

### Multiple Files, Multiple Entrypoints

By naming code blocks with prefix 'file:' followed by a relative path, multiple code files can be created
from one source file. Each code block with the 'file:' prefix is treated as a separate entry point.

~~~md
```rust - file:src/lib.rs
fn say_hello() {
    println!("Hello Literate Programmer!");
}
```
~~~

### File transclusion

Outline supports file transclusion to allow for more structured or modular project sources.
*This feature is currently only supported for Markdown.*

Here, the content of file `src/main.rs.md` would be pulled into the document
before compilation:

```md
@{{[src/main.rs](src/main.rs.md)}}
```
which would render as a link in the sources file. Or simply:
```md
@{{src/main.rs.md}}
```
which would not render as a link.

Unnamed entrypoints are renamed to file entrypoints during transclusion.
It is not required that transcluded files have their own entrypoints.

All code blocks from transcluded files are accessible from the *transcluding* file,
as well as from other transcluded files.

### Include linked files

Files linked from the main source document are included in the compilation process.
*This feature is currently only supported for Markdown.*

As an example, Outline would also compile the file `src/main.rs.md` here:

```md
* [src/main.rs](src/main.rs.md)
```

However, files are only included in the compilation if they are referenced by a *relative path*, and the file exists in that location.

### Multiple languages

Some documentation formats allow you to indicate the language that a code block is written in. In
fact, it is recommended that you always include the language when you write a code block,
particularly if multiple programming languages are used within the same document.

By properly labelling all code blocks, it is then possible to write a program in multiple
programming languages at once. Whether this is practical or not remains to be seen, but it is
supported nonetheless. By then supplying a language name on the command line, only code blocks in
that language are used when generating the tangled source. For example, here is a trivial program
written in two languages:

~~~tex
Here we have hello world in Ruby:

```rb
puts "Hello world"
```

And here it is again in Rust:

```rust
fn main() {
  println!("Hello world");
}
```
~~~

Compiling this with no language supplied with just ignore language information, so a single output
will be generated containing both languages. However, supplying the `--language rb` flag to Outline
will cause only the code blocks tagged with `rb` will be used to generate code.

## Usage

```
Literate programming compiler

USAGE:
    outline [OPTIONS] [input]... [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -o, --output <code_dir>          Output tangled code files to this directory. If none is specified, uses 'path' ->
                                     'code' from config file.
    -c, --config <config_file>       Sets the config file name [default: Outline.toml]
    -d, --docs <doc_dir>             Directory to output weaved documentation files to. If none is specified, uses
                                     'path' -> 'docs' from config file.
    -e, --entrypoint <entrypoint>    The named entrypoint to use when tangling code. Defaults to the unnamed code block.
    -l, --language <language>        The language to output the tangled code in. Only code blocks in this language will
                                     be used.
    -s, --style <style>              Sets the style to use. If not specified, it is inferred from the file extension.
                                     [possible values: bird, md, tex, html]

ARGS:
    <input>...    The input source file(s). If none are specified, uses 'path' -> 'files' from config file.

SUBCOMMANDS:
    create    Creates an outline project in the current directory
    help      Prints this message or the help of the given subcommand(s)
```

### Configuration

Each style supports some additional configuration, which is provided via a toml configuration file
(default: Outline.toml). Multiple styles can be configured at once in the configuration file. Note
that if a style appears in the configuration file, its full set of options is required (all defaults
will be discarded).

For more information on these options, see the [API documentation](https://docs.rs/outline).

```toml
[md]
fence_sequence = "```"
fence_sequence_alt = "~~~"
block_name_start = " - "
comments_as_aside = false
comment_start = "//"
interpolation_start = "@{"
interpolation_end = "}"
macro_start = "==> "
macro_end = "."
transclusion_start = "@{{"
transclusion_end = "}}"
variable_sep = ":"
file_prefix = "file:"
hidden_prefix = "hidden:"
blank_lines = true

[tex]
default_language = "rs" # optional
code_environment = "code"
comment_start = "//"
interpolation_start = "@{"
interpolation_end = "}"
macro_start = "==> "
macro_end = "."
variable_sep = ":"
file_prefix = "file:"
hidden_prefix = "hidden:"
blank_lines = true

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
variable_sep = ":"
file_prefix = "file:"
hidden_prefix = "hidden:"
blank_lines = true

[bird]
code_marker = "> "
code_name_marker = ">>> "
comment_start = "//"
interpolation_start = "@{"
interpolation_end = "}"
macro_start = "==> "
macro_end = "."
variable_sep = ":"
file_prefix = "file:"
hidden_prefix = "hidden:"
blank_lines = true
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

## Acknowledgement

This tool is derived from work of [foxfriends](https://github.com/foxfriends),
named [outline](https://github.com/foxfriends/outline).
