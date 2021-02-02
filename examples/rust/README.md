# Rust example

This examples demonstrates Yarner's most important features by creating a simple but complete [Rust](https://rust-lang.org) project.

For help on the Literate Programming syntax, see the [User Guide](https://mlange-42.github.io/yarner/).

To build the project (i.e. generate code and documentation output), run the following command in the current directory:

```
> yarner
```

## Readme

Every project should have a readme. We create a file `README.md` by using a code block named with prefix `file:`.

```markdown
//- file:README.md
# Rust project

This is a simple but complete Rust project with a `README.md` file,
a `Cargo.toml` file, a `.gitignore` file and some source code.
```

## Rust code - macros and multiple files

We create a file `src/main.rs`, again by using a code block named with prefix `file:`. The function declarations called in `main` are drawn from multiple code blocks named `Functions`.

```rust
//- file:src/main.rs
fn main() {
    start();
    for i in 0..10 {
        step(i);
    }
    stop();
}
// ==> Functions.
```

Function code blocks are here:

```rust
//- Functions
fn start() {
    println!("Starting");
}
```

Multiple code blocks of the same name are concatenated.

```rust
//- Functions
fn stop() {
    println!("Stopping");
}
```

```rust
//- Functions
fn step(step: i32) {
    println!("Step {}", step);
}
```

## Project files - transclusions and links

For a complete Rust project, we need a file `Cargo.toml` and a file `.gitignore`.

For the first one, we use file transclusions. In the complied documentation in sub-folder `docs`, The line below will be replaced by the content of the given file.

@{{[Cargo.toml.md](Cargo.toml.md)}}

Files linked with a certain prefix (`@` by default) are also included in the compilation process if the link is relative and the file exists.
By linking to file @[.gitignore.md](.gitignore.md), we use this feature to include it into processing. The prefixed is removed in documentation output.

## Reverse mode

This project is set up to enable Yarner's reverse mode. In the generated file `code/src/main.rs`, you will find comment lines that delineate code blocks. Do not delete or modify these lines. Except this limitation, you can modify the file, and afterwards play back changes into the documentation sources with

```
> yarner reverse
```

To get clean code output without block labels, run yarner with option `--clean`:

```
> yarner --clean
```
