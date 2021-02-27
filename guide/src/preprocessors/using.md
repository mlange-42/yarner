# Using pre-processors

This chapter explains how to use pre-processors.

[[_TOC_]]

## How it works

Pre-processors are external programs that can modify the documents parsed by Yarner. They communicate with Yarner using JSON through stdin/stdout.

## Configuration

A pre-processor program must be on the `PATH` to be usable.

To use a pre-processor for a project, add the respective section to the `Yarner.toml` file. E.g. for a pre-processor `yarner-block-links`, add

```toml
[preprocessor.block-links]
```

Pre-processor options follow after the secion:

```toml
[preprocessor.block-links]
join = " | "
```

Multiple pre-processors can be combined. They will process the document in the order they are given, each receiving the output of its precursor.

With these settings, run Yarner a usual.

Pre-processors have no effect in reverse mode.

## Command

By default, the command of the pre-processor is derived from its name in the config file, prefixed with `yarner-`. E.g., the command derived from `[preprocessor.block-links]` is `yarner-block-links`.

Alternatively, each pre-processor section can have an optional parameter `command`, e.g.

```toml
[preprocessor.xyz]
command = "path/to/binary"
```
