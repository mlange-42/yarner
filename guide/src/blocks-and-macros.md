# Blocks and macros

Yarner's aim is to make it possible to create software from documents written and structured for humans.
The most important features in that respect are (named) code blocks and macros.
This chapter provides an introduction to their basic usage.

[[_TOC_]]

## Code blocks

In the most basic scenario, Yarner extracts code blocks from a Markdown document and writes them to an equally-named file in the `code` output directoy, with the file's `md` extension stripped off.

As an example, create a project like this:

```plaintext
> yarner create main.rs.md
```

This would create a file `main.rs.md` in your project's directory, and a `Yarner.toml` file with the default settings.

File `main.rs.md` has some template content, but we want to start with a minimal example:

````markdown
# Simple example

The program's entry point:

```rust
fn main() {
    println!("Hello World!");
}
```
````

Running yarner with

```plaintext
> yarner
```

creates a file `main.rs` in sub-directory `code`, and a documentation file `main.rs.md` in sub-directory `docs`.
In this case, `docs/main.rs.md` has the same content as the original file, and `code/main.rs` contains the extracted code block:

```rust,noplaypen
fn main() {
    println!("Hello World!");
}
```

## Macros

To allow structuring for humans, Yarner uses macros. 
For that sake, code blocks can be given a name in their first line, by default prefixed with `//-`.
Here is a code block named `Say hello`:

````markdown
```rust
//- Say hello
fn say_hello() {
    println!("Hello World!");
}
```
````

During code extraction, code is drawn together by replacing macro invocations by the respective code block's content.
By default, a macro invocation starts with `// ==>` and ends with `.`.

````markdown
```rust
fn main() {
    say_hello();
}

// ==> Say hello.
```
````

As a complete example, the content of a file `main.rs.md` could look like this:

````markdown
# Simple example

The program's entry point:

```rust
fn main() {
    say_hello();
}
// ==> Say hello.
```

Here is how we say hello:

```rust
//- Say hello
fn say_hello() {
    println!("Hello World!");
}
```
````

The resulting content of `code/main.rs` would look like this:

```rust,noplaypen
fn main() {
    say_hello();
}
fn say_hello() {
    println!("Hello World!");
}
```

**Macro evaluation is recursive.** Thus, code blocks that are referenced by macros can also contain macro invocations.

## Named entrypoints

By default, unnamed code blocks are the entrypoints for code extraction.
This can be changed in the config file `Yarner.toml` through the option `entrypoint` in section `[paths]`:

```toml
...
[paths]
entrypoint = "Main"
...
```

The code block named by `//- Main` in its first line would then be used as entrypoint.

## Concatenated code blocks

Multiple code blocks with the same name are concatenated in their order of appearance in the source document.
As an example, here is an alternative content for `main.rs.md`:

````markdown
# Simple example

The program's entry point:

```rust
fn main() {
    say_hello();
}
```

Here is how we say hello:

```rust
fn say_hello() {
    println!("Hello World!");
}
```
````

The two code blocks would be concatenated and result in this content of `code/main.rs`:

```rust,noplaypen
fn main() {
    say_hello();
}
fn say_hello() {
    println!("Hello World!");
}
```
