# Meta variables

If you consider a macro invocation like a function call, then meta variables are like parameters.

[[_TOC_]]

## Meta variables

By default, to indicate that a macro includes a meta variable, the name of the variable must be part of the name of the macro, delimited by `@{` and `}`.
Then that meta variable may be used within the macro by again using its name within the `@{` and `}` in the code.

Finally, a macro with meta variables is invoked by replacing the name of the variable with its value in the invocation.

An example:

````markdown
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
````

## Default values

Meta variables can have default values:

````markdown
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
````

These features allow for more flexibility when writing macros, as well as possibly making the intent clearer.

However, macros and variables should not be abused to replace the mechanisms of abstraction provided by the target language. These are preferable as they enforce semantic in addition to purely syntactic structure.

## Limitations

Meta variables not work with [Reverse mode](./reverse-mode.md). When playing back from code output to sources, variable names in source macros are eliminated, while variables in code blocks are replaced by their values.

Both features can still be combined by using them only different languages. I.e. use macros only in languages for which section `block_labels` is not given in the `Yarner.toml`.
