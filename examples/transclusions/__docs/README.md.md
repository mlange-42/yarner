# Transclusion tests

Code for README:

```md
# Readme, generated by `yarner`

All code in this folder and subfolders, including this readme,
`Cargo.toml` and `.gitignore` were created using literate programming.

You find the documentation from which the code and all other files were derived from
here: [../README.md.md](../README.md.md).

==> Transcluded 1.

==> Transcluded 2.
```

Line 1 before transclusion


```md
// Transcluded 1
**This is transcluded**
```

Here comes a deep transclusion:


```md
// Transcluded 2
**This is transcluded twice**
```


Here comes a transclusion with entypoint (`main.rs`)

This goes to file `main.rs`

```md
// file:main.rs
fn main() {
    
}
```