# Reverse mode

Programming inside Markdown code blocks may be inconvenient due to missing syntax highlighting and the lack of IDE support in general.
Yarner offers a "reverse mode" that lets you edit generated code files, e.g. in an IDE, and to play back changes into the Markdown sources.

<div style="border: 2px solid red; padding: 0.5em;">
Warning: This feature is still experimental and modifies the original Markdown sources. Make a backup of the sources before using it!<br/>
</div>

[[_TOC_]]

## Usage

To use the reverse mode, run the following after making changes to generated code files:

```plaintext
> yarner --reverse
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

In most cases, only the option `comment_start` needs to be adapted to the line comment sequence of the target language. E.g., Python would require the following:

```toml
[language.py]
comment_start = "#"
...
```

Option `comment_end` is provided for languages that support only block comments and sould be left out in all other cases.

For details on the available options, see chapter [Languages](./config-languages.md).

Multiple languages can be defined by simply adding one section per language.
It is, however, not necessary to provide language settings for every file extension present.
Files with no language settings for their extension are simply ignored during reverse mode.

## Code block labels

Code in output intended for the reverse mode is labeled to allow Yarner to identify its file and code block of origin. You can edit everything between labels, but do not modify or delete the labels themselves!

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

With language settings for Rust as given above, the generated code in `main.rs` would look like this:

```rust,noplaypen
// <@main.rs.md#
fn main() {
    // <@main.rs.md#Say hello
    println!("Hello World!");
    // @>main.rs.md#Say hello
}
// @>main.rs.md#
```

## Clean code output

For clean code output without block labels, run Yarner with option `--clean`:

```plaintext
> yarner --clean
```

Of course, the reverse mode does not work with clean output.

## Limitations - comment extraction

The [Comment extraction](./comment-extraction.md) feature does not work with reverse mode.
As Yarner's special comments (`//-` by default) are not written to code files, they will be lost during reverse mode.

This may be fixed in a future version of Yarner.
