# Watch command

Yarner has a subcommand `watch` to automatically build a project after files changed.
This may be particularly convenient when switching back and forth
between Markdown editing and code editing with reverse mode.

<div style="border: 2px solid red; padding: 0.5em;">
Warning: This feature is still experimental and modifies the original Markdown sources.
Make a backup of the sources before using it!
</div>

[[_TOC_]]

## Usage

To start watching, run subcommand `watch`:

```plaintext
> yarner watch
```

Yarner will do one forward build and then watch the detected source files,
as well as the generated code files, for changes.
When source files change, Yarner runs a forward build.
When code files change, it runs a reverse build.

The config file `Yarner.toml` is watched as a source file,
thus a forward run will be done on changes.

To stop watching, press `Ctrl + C`.

## Reverse pass

In watch mode, a reverse build is immediately followed by a forward build,
but exclusively for the documentation output.
This way, code changes are immediately reflected in the documentation.

## Error handling

When watching a project, some errors are handled less strict
in order to not abort watching unnecessarily.
Instead of aborting with an error, a warning is printed.
In particular, this applies to plugin errors.
