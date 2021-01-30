# Yarner template

The following code goes to the base file of code output:

```md
Hello Literate Programmer!

// ==> More code.
```

In the code output, `==> More code.` will be replaced by the following code:

```md
//- More code
Have fun with yarner!
```

To create code in other files, use `file:<path/to/file>` as the block name.
Here, we create a file `main.rs` in subfolder `src`:

```rust
//- file:src/main.rs
fn main() {
    println!("Hello Literate Programmer!");
    // ==> More code in main.
}
```

Pulling code together works as usual:

```rust
//- More code in main
println!("Have fun with yarner!");
```
