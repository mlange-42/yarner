# Dead and hidden code

In some cases, it may be desirable to have code blocks that don't go into code output, or code that is hidden in documentation.

[[_TOC_]]

## Dead code

Code blocks like examples may be intended to be excluded from code output. 
This can be achieved by giving a block a name that is not used in any macro.
In this example, the code block named `Example` will be absent in the generated code as it is not referenced from anywhere:

````markdown
# Dead code example

A function to greet someone:

```rust
fn say_hello(name: &str) {
    println!("Hello {}", name);
}
```

Function `say_hello_to` can be used like that:

```rust
//- Example
fn say_hello("Paul");
```
````

Additionally, if the variable `entrypoint` in section `[paths]` of the `Yarner.toml` is set, unnamed blocks are excluded from code output.

## Hidden code

Sometines, it can be useful to exclude code that is of limited interest for the reader from documentation output.
This can be achieved by prefixing block names with `hidden:` (the default, configurable):

````
```rust
//- hidden:A hidden function
fn hidden() {
    
}
```
````

For code output, hidden blocks are treated like regular code blocks.

Only named code blocks can be hidden.
