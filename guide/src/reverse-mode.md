# Reverse mode

Programming inside Markdown code blocks may be inconvenient due to missing syntax highlighting and the lack of IDE support in general.
Yarner offers a "reverse mode" that lets you edit generated code files, e.g. in an IDE, and to play back changes into the Markdown sources.

[[_TOC_]]

## Usage

To use the reverse mode, run the following after making changes to generated code files:

```plaintext
> yarner reverse
```

Reverse mode requires settings for the target language(s) to be defined. See the following section.

## Language settings

To enable the reverse mode, Yarner needs to label code blocks in the generated sources to identify their origin.

The `Yarner.toml` file provides a section where settings for multiple languages can be specified. The language is determined from the extension of the output file. The following example provides settings for Rust that would be applied to all `.rs` files.

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

In most cases, only the option `comment_start` needs to be adapted to the line comment sequence of the target language. E.g., Python requires the following:

```toml
[language.py]
comment_start = "#"
...
```

Option `comment_end` is provided for languages that support only block comments and should be left out in all other cases.

For details on the available options, see chapter [Languages](./config-languages.md).

Multiple languages can be defined by simply adding one section per language.
It is, however, not necessary to provide language settings for every file extension present.
Files with no language settings for their extension are simply ignored during reverse mode.

## Code block labels

Code in output intended for the reverse mode is labelled to allow Yarner to identify its file and code block of origin. You can edit everything between labels, but do not modify or delete the labels themselves!

As an example, a simple Markdown source file `main.rs.md` could have the following content:

````markdown
# Simple example

The program's entry point:

```rust
fn main() {
    // ==> Say hello.
}
```

Here is how we say hello:

```rust
//- Say hello
println!("Hello World!");
```
````

With language settings for Rust as given above, the generated code in `main.rs` looks like this:

```rust,noplaypen
// <@main.rs.md##0
fn main() {
    // <@main.rs.md#Say hello#0
    println!("Hello World!");
    // @>main.rs.md#Say hello#0
}
// @>main.rs.md##0
```

## Copied files

If files were copied as explained in chapter [Copying files](./copying-files.md), Yarner detects these in reverse mode and copies them back. I.e. code in copied files can be modified just like code extracted from code blocks, but without the need to care for block labels.

## Lock file

When reverse mode for a project is enabled (by providing the required language settings), a file `Yarner.lock` is created in the project's `root`.
The file is required to prevent accidental overwrites of user edits in Markdown sources as well as code output.
E.g., after editing the code output, Yarner will refuse to do a forward build as your changes would then be lost.
To build the project nonetheless, run with option `--force`:

```plaintext
> yarner --force
> yarner --force reverse
```

The file `Yarner.lock` should be ignored by Version Control Systems (i.e. add `Yarner.lock` to your `.gitignore`).

## Clean code output

For clean code output without block labels, run Yarner with option `--clean`:

```plaintext
> yarner --clean
```

Of course, the reverse mode does not work with clean output.

## Limitations

### Block repetitions

When the same code block is use by multiple macro invocations, it is ambiguous which one to play back into the sources. Here is an example:

```rust,noplaypen
fn main() {
    // ==> Say hello.
    // ==> Say hello.
}
```

In such cases, Yarner emits a warning when called with subcommand `reverse`. If the occurrences differ, like in the following example of user-modified code output, it aborts with an error.

```rust,noplaypen
// <@main.rs.md#
fn main() {
    // <@main.rs.md#Say hello#0
    println!("Hello World!");
    // @>main.rs.md#Say hello#0
    // <@main.rs.md#Say hello#0
    println!("Hello Universe!");
    // @>main.rs.md#Say hello#0
}
// @>main.rs.md#
```
