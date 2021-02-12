# mdBook with Yarner

[mdBook](https://github.com/rust-lang/mdBook) is a command line tool to create online books from Markdown files (e.g. the book you are currently reading). This chapter explains how to use it together with Yarner.

[[_TOC_]]

## Initialization

To create a project for mdBook and Yarner, run both with their sub-command `init`:

```plaintext
> yarner init
> mdbook init
```

Delete the file `README.md` created by Yarner (or better, fill it with a readme for the project). Further, the correct paths need to be set in file `Yarne.toml`. See the next section.

## Settings

Change section `[paths]` in file `Yarner.toml` to use these options:

```toml
[parser]
...

[paths]
root = "lp/"
code = "../code/"
docs = "../src/"

files = ["SUMMARY.md"]
...
```

## Project structure

The recommended structure as resulting from the above initialization and settings looks like this (some directories are initially missing):

```plaintext
project
  |
  |-- book/        <-------.       <rendered book>
  |                        |
  |-- code/                |       <code output>
  |     '-- ...         <--|--.
  |                        |  |
  |-- src/                 |  |
  |     |-- SUMMARY.md  ---'  |    <doc output/book sources>
  |     '-- capter-1.md <-----|
  |                           |
  |-- lp/                     |
  |     |-- SUMMARY.md  ------'    <yarner sources>
  |     '-- capter-1.md
  |
  |-- book.toml
  '-- Yarner.toml
```

Directory `lp` contains the Markdown source files. Write these files as you would normally write mdBook files in directory `src`. The only difference is that entries in `SUMMARY.md` that contain Literate Programming code are prefixed with `@` (for link-following). As an example, `lp/SUMMARY.md` could look like this:

```markdown
# Summary

* @[Chapter 1](./chapter-1.md)
* @[Chapter 2](./chapter-2.md)
```

From the Markdown sources in `lp`, Yarner creates files for mdBook in directory `src`, as well as extracted code in directory `code`:

```plaintext
> yarner
```

Finally, mdBook uses the files in `src` to create the HTML website in directory `book`:

```plaintext
> mdbook build
```
