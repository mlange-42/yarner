# Copying files

Sometimes it may be desired to include code that is not part of the Markdown sources.
Further, it may be necessary to include files in the documentation that are not processed by Yarner, like images.
For such cases, Yarner can automatically copy files into code and documentation output.

[[_TOC_]]

## Copying files

Section `[paths]` of a project's `Yarner.toml` provides options to list files and patterns for copying files unchanged, `code_files` and `doc_files`. Both accept a list of file names or glob patterns.

As an example, with the following setting Yarner would copy all `.png` and `.jpg` files from directoy `img` to `docs/img`:

```toml
doc_files = ["img/*.png", "img/*.jpg"]
```

Equivalently, one could automatically copy all Rust files from `src` to `code/src` like this:

```toml
code_files = ["src/*.rs"]
```

## Modifying paths

In some cases it may be inconvenient to be forces to equal structure in sources and outputs.
Through options `code_paths` and `doc_paths`, paths can be modified to some extent during processing.

Currently, only replacement of path components as well as omission of components are supported.

As an example, some files may be required to end up at the top level directory of the code output, but should not be at the top level of sources. In the following example, all files and folders from directory `additional-code` would be copied directly into the `code` output folder:

```toml
code_files = ["additional-code/**/*"]
code_paths = ["-"]
```

When present, options `code_paths` and `doc_paths` must have as many entries as `code_files` and `doc_files`, respectively.
Each entry of e.g. `code_paths` is applied to the corresponding entry in `code_files`.

**Possible modifications:**

* Use `foo/bar` to replace the first two path component by `foo` and `bar`.
* Use `-` (minus) to ommit, and `_` (underscore) to preserve a component.
* Use a single `_` (underscore) if no path change is intended at all.
