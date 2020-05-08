# Markdown outline template

The following code goes to the base file of code output:

```
Hello Literate Programmer!

==> More code.
```

In the code output, `==> More code.` will be replaced by the following code:

```
// More code
Have fun with outline!
```

To create code in other files, use `file:<path/to/file>` as block name.
Here, we create a file `main.rs` in subfolder `src`:

```rust
// file:src/main.rs
fn main() {
    println!("Hello Literate Programmer!");
    ==> More code in main.
}
```

Pulling code together works as usual:

```rust
// More code in main
println!("Have fun with outline!");
```

## Alternative block name syntax

This template uses the alternative block name syntax using comments,
where blocks in the rendered source file look exactly like in the rendered docs output.

Alternatively, use this syntax, where block names are not visible in the rendered sources:
```md - Code block syntax
~~~rust - More code in main
println!("Have fun with outline!");
~~~
```

(Replace ` ~~~ ` with ` ``` `.)

When using this syntax, the comment syntax used above is ignored and treated as normal code.
