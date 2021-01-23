# Features

Features in Markdown code blocks:

* [Macros](#Macros)
* [Named entrypoints](#named-entrypoints)
* [Comment extraction](#extracted-comments)
* [Multiple files, multiple entrypoints](#multiple-files-multiple-entrypoints)
* [File transclusions](#file-transclusions)
* [Include linked files](#include-linked-files)
* [Dead and hidden code blocks](#dead-and-hidden-code-blocks)
* [Multiple languages in one file](#multiple-languages)
* [Meta variables](#meta-variables)

The text sections simply copied in and written out with no processing, except for file transclusions and comment extraction. This allows you to write your documentation however you like. Note that any post-processing like conversion from Markdown to HTML should be performed on the generated document(s).

## Macros

Macros are what enables the literate program to be written in logical order for the human reader.
Using Yarner, this is accomplished by naming the code blocks, and then referencing them later by
"invoking" the macro.

The code block name is given by the text in the first line of the block if it starts with the comment sequence (default `//-`, see [Comment extraction](#extracted-comments) for details).

By default, macro invocations start with `// ==>` and end with a period `.`. Both of these
sequences can be customized to suit your needs better. The only restriction with macro invocations
is that they must be the only thing on the line.

```rs
fn main() {
  // ==> Calculate the very complex result.
  // ==> Print the results for the user.
}
```

If two code blocks have the same name, they are concatenated, in the order they are written. This can be very useful in defining global variables or listing imports closer to the parts where they are used.

## Named entrypoints

By default, the entrypoint of the program is always the unnamed code block. However, this limits the
output of one input file to always be the same source code. It also means that you can't have a name
on the entrypoint in the documentation, which could be useful.

To get around this, an entrypoint name can be passed to Yarner on the command line. Then, instead
of starting at the unnamed code block, it will start at the code block with this name.

Note that if you use a named entrypoint, there is no way to reference the unnamed code blocks as
macros. You can, however, use the unnamed code blocks to provide examples, for example, to the
readers of the documentation, so they are still useful.

## Extracted comments

By default, the comment extraction sequence is set to `//-` for familiarity, but also distiguish it from the sequence `//` used by many languages for comments while playing nicely with Markdown syntax highlighting.

Any text after (and including) this sequence is extracted from the code block, and not rendered to the tangled source code. Note that, since the comments are removed completely when compiling, they do not have to use the actual line comment indicator from you programming language. In fact, it may be better to choose a sequence that is *not* the regular comment indicator so that you can still have comments in your tangled code output (as with the default `//-`).

Now that these comments are extracted, it is possible to handle them differently in the weaved documentation file. In particular, extracted comments can be rendered in `<aside>` tags, which can then be rendered nicely using CSS.

## Multiple Files, Multiple Entrypoints

By naming code blocks with prefix `file:` followed by a relative path, multiple code files can be created from one source file. Each code block with the `file:` prefix is treated as a separate entry point.

~~~md
```rust
//- file:src/lib.rs
fn say_hello() {
    println!("Hello Literate Programmer!");
}
```
~~~

## File Transclusions

Yarner supports file transclusion to allow for more structured or modular project sources.

Here, the content of file `src/main.rs.md` would be pulled into the document before compilation:

```md
@{{[src/main.rs](src/main.rs.md)}}
```

which would render as a link in the sources file. Or simply:

```md
@{{src/main.rs.md}}
```
which would not render as a link.

Unnamed entrypoints are renamed to file entrypoints during transclusion. As an example, including a file `main.rs.md` with the following content:

~~~md
Main function:

```rust
fn main() {
    println!("Hello Literate Programmer!");
}
```
~~~

produces in the following content in the including document:

~~~md
Main function:

```rust
//- file:main.rs
fn main() {
    println!("Hello Literate Programmer!");
}
```
~~~

It is not required that transcluded files have their own entrypoints. All code blocks from transcluded files are accessible from the *transcluding* file, as well as from other *transcluded* files.

## Include linked files

Files linked with a certain prefix (`@` by default) are included in the compilation process. As an example, Yarner would also compile the file `src/main.rs.md` here:

```md
* @[src/main.rs](src/main.rs.md)
```

However, files are only included in the compilation if they are referenced by a *relative path*, and the file exists in that location. The prefix is removed before writing documentation output.

## Dead and hidden code blocks

Code bocks that are never referenced by a macro are not "compiled" into code. This fact can be use to create "dead" code blocks, e.g. to provide examples that are not part of the target sources. For an example, see below.

To hide a code blocks in the documentation output, prefix the name with `hidden:` (by default). The `hidden:` prefix does, however, not become part of the name, so some of the blocks with the same name can be shown, while others are hidden.

Example:

~~~md
There is a struct with uninteresting implementation:

```rust
// ==> Position.
```

The `Position` struct:

```rust
//- Position
struct Position {
    pub x: f64,
    pub y: f64
}
```

Implementation details are omitted for the sake of brevity.

```rust
//- hidden:Position
impl Position {
    fn zero() -> {
        Position { x: 0, y: 0 }
    }
}
```

This usage example does not appear in the code:

```rust
//- Example
let p = Position { x: 100, y: 100 };
```
~~~

Yarner produces the following code, not containing the example code block:

```rust
struct Position {
    pub x: f64,
    pub y: f64
}

impl Position {
    fn zero() -> {
        Position { x: 0, y: 0 }
    }
}
```

In the generated documentation, the implementation of struct `Position` is hidden:

---

There is a struct with uninteresting implementation:

```rust
// ==> Position.
```

The `Position` struct:
```rust
//- Position
struct Position {
    pub x: f64,
    pub y: f64
}
```

Implementation details are omitted for the sake of brevity.

This usage example does not appear in the code:

```rust
//- Example
let p = Position { x: 100, y: 100 };
```

---

## Multiple languages

Markdown allow you to indicate the language that a code block is written in. In fact, it is recommended that you always include the language when you write a code block, particularly if multiple programming languages are used within the same document.

By properly labelling all code blocks, it is then possible to write a program in multiple programming languages at once. By supplying a language name on the command line, only code blocks in that language are used when generating the tangled source. For example, here is a trivial program written in two languages:

~~~md
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

Compiling this with no language supplied will just ignore language information, so a single output will be generated containing both languages. However, supplying the `--language rb` flag to Yarner will cause only the code blocks tagged with `rb` will be used to generate code.

## Meta variables

If you consider a macro invocation like a function call, then meta variables are like parameters.

By default, to indicate that a macro includes a meta variable, the name of the variable must be part of the name of the macro, delimited by `@{` and `}`.

Then that meta variable may be used within the macro by again using its name within the `@{` and `}` in the code.

Finally, a macro with meta variables is invoked by replacing the name of the variable with its value in the invocation.

An example:

~~~md
Here is our macro with meta variables:

```rust
//- Say @{something} to @{someone}
println!("Hey, @{someone}! I was told to tell you \"@{something}\"");
```

Now, to say things to many people:

```rust
// ==> Say @{Hello} to @{Jim}.
// ==> Say @{How are you} to @{Tom}.
// ==> Say @{I am good!} to @{Angela}.
```
~~~

Meta variables can have default values:

~~~md
Here is our macro with default meta variables:

```rust
//- Say @{something:Hello} to @{someone}
println!("Hey, @{someone}! I was told to tell you \"@{something}\"");
```

Now, to say the default "Hello" to many people:

```rust
// ==> Say @{} to @{Jim}.
// ==> Say @{} to @{Tom}.
// ==> Say @{} to @{Angela}.
```
~~~

These features allow for more flexibility when writing macros, as well as possibly making the intent clearer.

However, macros and variables should not be abused to replace the mechanisms of abstraction provided by the target language. These are preferable as they enforce semantic in addition to purely syntactic structure.
