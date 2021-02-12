# Project structure

Yarner offers flexibility regarding the structure of projects. This chapter presents project layouts recommended for different use cases.

[[_TOC_]]

## Settings

For an explanation of all settings that serve to configure the project structure, see chapter [Paths](../config-paths.md). The most important settings are:

```toml
[paths]
root = "."
code = "code/"
docs = "docs/"

files = ["README.md"]
```

`root` is the path all other paths are relative to. `code` is for code output, and `docs` for documentation output. `files` are the entrypoint files of the project (normally one, but can be more, or even glob patterns).

## Default structure

The default setup created by `init` uses the setting listed above and produces this project structure:

```plaintext
project
  |
  |-- code/              <code output>
  |-- docs/              <doc output>
  |
  |-- README.md          <source document>
  '-- Yarner.toml        <config>
```

The structure is well suited for small projects that use only a single source file and no marked links (`@`) or transclusions. In this case, the documentation output looks exactly like the sources. The source file (e.g. `README.md`) can thus be used as the documentation directly, and directory `docs` can be ignored by Git (in file `.gitignore`):

```plaintext
/docs/
```

## Three folders structure

Another possible structure is to have the Markdown sources in a separate sub-folder, e.g. `lp`:

```plaintext
project
  |
  |-- code/              <code output>
  |-- docs/              <doc output>
  |-- lp/                <source files>
  |     |-- README.md
  |     '-- ...
  |
  '-- Yarner.toml        <config>
```

The required settings look like this:

```toml
[paths]
root = "lp/"
code = "../code/"
docs = "../docs/"

files = ["README.md"]
```

This layout is suitable for larger projects with potentially many linked or transcluded source files.

Here, an additional file `README.md` that contains no Literate Programming code can be placed at the top level of the project. The compiled documentation for readers (after transclusion, with clean links) can be found in directory `docs`.

## Top-level documentation structure

A further useful layout is to place the documentation output at the top project level, while the sources are in a sub-folder (here, `lp`):

```plaintext
project
  |
  |-- code/              <code output>
  |-- lp/                <source files>
  |     |-- README.md
  |     '-- ...
  |
  '-- Yarner.toml        <config>
```

The required settings look like this:

```toml
[paths]
root = "lp/"
code = "../code/"
docs = "../"

files = ["README.md"]
```

Here, the "compiled" documentation output (e.g. `README.md`) is placed directly in the project directory and thus presented to the reader.

This structure is useful for larger projects that use link following and transclusions, but still want the Literate Programming document directly presented to the reader, e.g. as repository `README.md`.
