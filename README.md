# Yarner

[![Build Status](https://travis-ci.com/mlange-42/yarner.svg?branch=master)](https://travis-ci.com/mlange-42/yarner)

A language-independent [Literate Programming](https://en.wikipedia.org/wiki/Literate_programming) tool for Markdown. From Markdown documents written and structured for humans, Yarner extracts code blocks into compilable source code. It offers sufficient features and flexibility to be usable also for larger projects with numerous files and multiple languages.

Yarner works with familiar syntax, which can be further customized to suit your needs exactly.
See the [examples](examples) directory for full working examples.
See the [**User Guide**](https://mlange-42.github.io/yarner/) for documentation.

## Features

* Macros and named code blocks
* Multiple files, multiple entrypoints
* File transclusions
* Reverse mode
* ...

See the [**User Guide**](https://mlange-42.github.io/yarner/) for a complete and detailed explanation of all features.

## Installation

**Pre-compiled binaries**

1. Download the [latest binaries](https://github.com/mlange-42/yarner/releases) for your platform
2. Unzip somewhere
3. *Optional:* add directory `yarner` to your `PATH` environmental variable

**Using `cargo`**

In case you have [Rust](https://www.rust-lang.org/) installed, you can install with `cargo`:

```plaintext
> cargo install --git https://github.com/mlange-42/yarner
```

## Getting started

To set up a new project, use the `init` sub-command. Run the following in your project's base directory:

```plaintext
> yarner init
```

This creates a file `Yarner.toml` with default settings, and a file `README.md` as starting point for Literate Programming.

The generated file already contains some content to get started with Yarner's basic features. For details, see the [User Guide](https://mlange-42.github.io/yarner/).

To build the project (extract code and create documentation), simply run:

```plaintext
> yarner
```

This creates two sub-directories, one containing the extracted code (a minimal but working Rust project), and another containing the final documentation.

Note that the contents of these directories can then be treated as usual, i.e. compiling the code with the normal compiler, or rendering Markdown to HTML or PDF.

## Examples

### Macros

Macros are what enables the literate program to be written in logical order for the human reader. Using Yarner, this is accomplished by naming the code blocks, and then referencing them later by "invoking" the macro.

By default, macro invocations start with `// ==>` and end with a period `.`. Both of these sequences can be customized to suit your needs better. The only restriction with macro invocations is that they must be the only thing on the line.

Here, we have an unnamed code block as entrypoint, and "draw" code from two other code blocks into the main function. These code blocks are named by their first line of code, starting with `//-`.

~~~markdown
The program starts in the main function. It calculates something and prints the result:

```rust
fn main() {
    // ==> Calculate something.
    // ==> Print the result.
}
```

The calculation does the following:

```rust
//- Calculate something
let result = 100;
```

Printing the result looks like this:

```rust
//- Print the result
println!("{}", result);
```
~~~

The rendered document looks like this:

<table><tr><td>

The program starts in the main function. It calculates something and prints the result:

```rust
fn main() {
    // ==> Calculate something.
    // ==> Print the result.
}
```

The calculation does the following:

```rust
//- Calculate something
let result = 100;
```

Printing the result looks like this:

```rust
//- Print the result
println!("{}", result);
```

</td></tr></table>

The generated code looks like this:

```rust
fn main() {
    let result = 100;
    println!("{}", result);
}
```

### Entrypoints

By default, the entrypoint of the program is always the unnamed code block. 
However, a code block name can be given in `Yarner.toml` or passed to Yarner on the command line. 
Then, instead of starting at the unnamed code block, it will start at the code block with this name.

By naming code blocks with prefix `file:` followed by a relative path, multiple code files can be created
from one source file. Each code block with the `file:` prefix is treated as a separate entry point.

~~~markdown
```rust
//- file:src/lib.rs
fn say_hello() {
    println!("Hello Literate Programmer!");
}
```
~~~

[File transclusions and links](https://mlange-42.github.io/yarner/links-and-transclusions.html) are further features that allow for projects with multiple code, documentation and/or source files.

## Configuration

Configuration is provided via a toml configuration file (default: `Yarner.toml`).
A file with default configurations is generated through the `init` sub-command.
See the comments in these files or user guide chapters on [configuration](https://mlange-42.github.io/yarner/configuration.html) for details on individual settings.
It is also the place to modify Yarner's syntax to suite your needs and preferences.

## Acknowledgements

This tool is derived from [foxfriends](https://github.com/foxfriends)' work [outline](https://github.com/foxfriends/outline).