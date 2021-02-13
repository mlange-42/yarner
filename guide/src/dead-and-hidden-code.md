# Dead and hidden code

In some cases, it may be desirable to have code blocks that don't go into code output, or code that is hidden in the documentation.

[[_TOC_]]

## Dead code

Code blocks like examples may be intended to be excluded from code output. This can be achieved by giving a block a name that is not used in any macro. In this example, the code block named `Example` will be absent from the generated code as it is not referenced from anywhere:

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

Additionally, if the option `entrypoint` in section `[paths]` of the `Yarner.toml` is set, unnamed blocks are excluded from code output. This can be useful to ignore e.g. simple command line usage examples intended to instruct the reader rather than for code output.

## Hidden code

Sometimes, it can be useful to exclude code that is of limited interest for the reader from documentation output. This can be achieved by prefixing block names with `hidden:` (the default, configurable):

````
```rust
//- hidden:A hidden function
fn hidden() {

}
```
````

For code output, hidden blocks are treated like regular code blocks.

Only named code blocks can be hidden.

The prefixes `hidden:` and `file:` can be combined, but only in that order:

````
```rust
//- hidden:file:secrets.rs
fn hidden() {

}
```
````

> Also note the features for [Links and transclusions](./links-and-transclusions.md) and for [Copying files](./copying-files.md). It is not necessary to have all code in the main document, nor to have it in Markdown code blocks at all.
