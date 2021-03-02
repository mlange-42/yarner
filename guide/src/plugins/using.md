# Using plugins

This chapter explains how to use plugins.

[[_TOC_]]

## How it works

Plugins are external programs that can modify the documents parsed by Yarner. They communicate with Yarner using JSON through stdin/stdout.

## Configuration

A plugin program must be on the `PATH` to be usable.

To use a plugin for a project, add the respective section to the `Yarner.toml` file. E.g. for a plugin `yarner-block-links`, add

```toml
[plugin.block-links]
```

Plugin options follow after the section:

```toml
[plugin.block-links]
join = " | "
```

Multiple plugins can be combined. They will process the document in the order they are given, each receiving the output of its precursor.

With these settings, run Yarner a usual.

Plugins have no effect in reverse mode.

## Command

By default, the command of the plugin is derived from its name in the config file, prefixed with `yarner-`. E.g., the command derived from `[plugin.block-links]` is `yarner-block-links`.

Alternatively, each plugin section can have an optional parameter `command`, e.g.

```toml
[plugin.xyz]
command = "path/to/binary"
```
