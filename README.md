# Yarner

[![Build Status](https://travis-ci.com/mlange-42/yarner.svg?branch=master)](https://travis-ci.com/mlange-42/yarner)

Generic literate programming transpiler. This project aims to provide a modern, developer friendly
literate programming tool.

Yarner works with familiar syntax, which can be further customized to suit your needs
exactly. It uses pluggable, configurable input formats, with out-of-the-box support for:
*   Markdown
*   Latex
*   HTML

See the [examples](examples) directory for full working examples in each style.

## Installation

**Pre-compiled binaries**

1. Download the [latest binaries](https://github.com/mlange-42/yarner/releases) for your platform
2. Unzip somewhere
3. *Optional:* add directory `yarner` to your `PATH` environmental variable

**Using `cargo`**

In case you have [Rust](https://www.rust-lang.org/) installed, you can install with `cargo`:

```
cargo install --git https://github.com/mlange-42/yarner
```

## Getting started

To set up a new project, use the `create` sub-command.
To create a Markdown project, run the following
in your project's base directory:

```
yarner create README.md md
```

Or simply, since Markdown is the default:

```
yarner create README.md
```

This creates a file `Yarner.toml` with default settings,
and a file `README.md.md` as starting point for Literate Programming
(don't care for the double extension for now).

The generated file already contains some content to get started with Yarner's
basic features. For details, read the following sections.

To "compile" the project (extract code and create documentation),
simply run:

```
yarner
```

This creates two sub-directories, one containing the extracted code,
and the other containing the final documentation.

Note that the contents of these directories can then be treated as usual,
i.e. compiling the code with the normal compiler,
or rendering Markdown to HTML or TeX to PDF.

> Note: You can move or copy the Yarner executable into your project directory for convenience.
> Otherwise, you need to specify the path to Yarner in the command, or add it to the PATH environment variable.

## Features

In all styles, the code sections are handled the same way, supporting:
* macros
* meta variable interpolation
* comment extraction
* named entrypoints
* multiple files, multiple entrypoints
* multiple languages in one file
* file transclusions

See [docs/Features.md](docs/Features.md) for a complete list and detailed explanation of all features.

Text sections are also handled the same way in all styles - just copied in and written out with
no processing. This allows you to write your documentation however you like.

## Examples

### Macros

Macros are what enables the literate program to be written in logical order for the human reader.
Using Yarner, this is accomplished by naming the code blocks, and then referencing them later by
"invoking" the macro.

By default, macro invocations start with a long arrow `==>` and end with a period `.`.
Both of these sequences can be customized to suit your needs better.
The only restriction with macro invocations is that they must be the only thing on the line. 

Here, we have an unnamed code block as entrypoint, and "draw" code from two other code blocks into the main function. These code blocks are named by their first line of code, starting with `//`.
~~~
The program starts in the main function. It calculates something and prints the result:
```rust
fn main() {
    ==> Calculate something.
    ==> Print the result.
}
```

The calculation does the following:
```rust
// Calculate something
let result = 100;
```

Printing the result looks like this:
```rust
// Print the result
println!("{}", result);
```
~~~

The rendered document looks like this:

----

The program starts in the main function. It calculates something and prints the result:
```rust
fn main() {
    ==> Calculate something.
    ==> Print the result.
}
```

The calculation does the following:
```rust
// Calculate something
let result = 100;
```

Printing the result looks like this:
```rust
// Print the result
println!("{}", result);
```
----

The generated code looks like this:

```rust
fn main() {
    let result = 100;
    println!("{}", result);
}
```

A feature to note is that if two code blocks have the same name, they are concatenated, in the order they are written. This can be very useful in defining global variables or listing imports closer to the parts where they are used.

### Entrypoints

By default, the entrypoint of the program is always the unnamed code block. However, a code block name can be passed to Yarner on the command line. Then, instead
of starting at the unnamed code block, it will start at the code block with this name.

By naming code blocks with prefix `file:` followed by a relative path, multiple code files can be created
from one source file. Each code block with the `file:` prefix is treated as a separate entry point.

~~~md
```rust
// file:src/lib.rs
fn say_hello() {
    println!("Hello Literate Programmer!");
}
```
~~~

[File transclusions](docs/Features.md#file-transclusions) and [Links](docs/Features.md#include-linked-files) are further features that allow for projects with multiple code, documentation and/or source files.

## Configuration

Each style supports some additional configuration, which is provided via a toml configuration file
(default: Yarner.toml). A file with default configurations is generated through the `create` sub-command.
See the comments in these files for details on individual settings.
This is also the place to modify Yarner's syntax to suite your needs and preferences.

## Usage

Most command line options can be specified in the project's `Yarner.toml` config file for convenience.
Command line options override options from the config file.

```
Literate programming compiler
  https://github.com/mlange-42/yarner

The normal workflow is:
 1) Create a project with
    > yarner create README.md
 2) Process the project by running
    > yarner

USAGE:
    yarner [FLAGS] [OPTIONS] [input]... [SUBCOMMAND]

FLAGS:
    -C, --clean      Produces clean code output, without block label comments.
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -o, --output <code_dir>          Output tangled code files to this directory. If none is specified, uses 'path' ->
                                     'code' from config file.
    -c, --config <config_file>       Sets the config file name [default: Yarner.toml]
    -d, --docs <doc_dir>             Directory to output weaved documentation files to. If none is specified, uses
                                     'path' -> 'docs' from config file.
    -e, --entrypoint <entrypoint>    The named entrypoint to use when tangling code. Defaults to the unnamed code block.
    -l, --language <language>        The language to output the tangled code in. Only code blocks in this language will
                                     be used.
    -s, --style <style>              Sets the style to use. If not specified, it is inferred from the file extension.
                                     [possible values: md, tex, html]

ARGS:
    <input>...    The input source file(s) as glob pattern(s). If none are specified, uses 'path' -> 'files' from
                  config file.

SUBCOMMANDS:
    create    Creates a yarner project in the current directory
    help      Prints this message or the help of the given subcommand(s)
```

## Acknowledgements

This tool is derived from [foxfriends](https://github.com/foxfriends)'
work [outline](https://github.com/foxfriends/outline).