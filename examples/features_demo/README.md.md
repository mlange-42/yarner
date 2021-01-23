# Yarner feature demo

This examples demonstrates Yarner's most important features by creating a simple but complete [Rust](https://rust-lang.org) project.

## Readme - unnamed entrypoint

Every project should have a readme. Here, we use an unnamed code block to create this file. Unnamed code blocks go to a file with the same name as the containing file, but with the `md` extensions removed. Here, `README.md.md` becomes `README.md`.

```markdown
# Rust project

This is a simple but complete Rust project with a `README.md` file,
a `Cargo.toml` file, a `.gitignore` file and some source code.
```

## Rust code - macros and multiple files

We create a file `src/main.rs` by using a code block named with prefix `file:`. The function declarations called in `main` are drawn from multiple code blocks named `Functions`.

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

Linked files are also included in the compilation process if the link is relative and the file exists. By linking to file [.gitignore.md](.gitignore.md), we use this feature to include it into processing.
