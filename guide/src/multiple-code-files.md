# Multiple code files

For most software projects, a single code file is not sufficient. Yarner provides several features for that purpose.
The most basic, producing multiple code files from a single Markdown document, is described here.

[[_TOC_]]

## Multiple files from a single document

It is possible to generate multiple code files from a single documentation source file through code blocks named with file paths, prefixed with `file:` (the default, configutable).

Besides `main.rs`, a file `main.rs.md` with the following content would also create a file `src/lib.rs`:

````markdown
# Multiple files example

File main.rs looks like this:

```rust
fn main () {

}
```

And here is the content of file `lib.rs`:

```rust
//- file:src/lib.rs
fn first_funtion() {}
fn second_funtion() {}

// ==> Further functions.
```

The remaining functions in `lib.rs`:

```rust
//- Further functions
fn third_funtion() {}
fn fourth_funtion() {}
```
````

Not that macro invocations are possible as usual, with no special syntax required.

Each code block named with the `file:` prefix is treated as separate entrypoint during the build process.

## Further uses of the feature

This feature can also be used to avoid the somewhat uncommon file naming patterns that were used in this guide so far. We generated code files from source files of the same name, but with an additional `md` extension. With the `file:` prefix feature, it is possible to circumvent this restriction completely.

As an example, it may be desired that the primary documentation file is named `README.md` (because the project is hosted on GitHub or GitLab), but to create a file `main.rs` from it. A file `README.md` with the following content would achieve that:

````markdown
# Simple example

The program's entry point:

```rust
//- file:main.rs
fn main() {
    println!("Hello World!");
}
```
````

Thus, file naming in documentation and code can be completely independent from each other.
