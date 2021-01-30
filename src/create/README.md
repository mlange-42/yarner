# Yarner template

This simple example creates a minimal but complete Rust project.

## File `src/main.rs`

To create code in a certain files, use `file:<path/to/file>` as the block name.
First, we create a file `main.rs` in subfolder `src`:

```rust
//- file:src/main.rs
fn main() {
    println!("Hello Literate Programmer!");
    // ==> More code in main.
}
```

The macro `// ==> More code in main.` pulls the code from the next code block into function `main`.

```rust
//- More code in main
println!("Have fun with yarner!");
```

## File `Cargo.toml`

For a complete Rust project, we also need a `Cargo.toml` file:

```toml
//- file:Cargo.toml
[package]
name = "hello-yarner"
version = "0.1.0"
authors = ["Your Name <you@example.com>"]
edition = "2018"
```

## Output

After running the command `yarner` in the project directory, extracted code can be found in sub-directory `code`, while documentation files are placed in `docs`.
