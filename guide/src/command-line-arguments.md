# Command line arguments

Some configurations can be overwritten using command line arguments.

[[_TOC_]]

## CLI help

To get help on command line options, use `yarner -h`.

```plaintext
Literate Programming tool for Markdown
  https://github.com/mlange-42/yarner

The normal workflow is:
 1) Create a project with
    > yarner init
 2) Process the project by running
    > yarner

USAGE:
    yarner [FLAGS] [OPTIONS] [FILES]... [SUBCOMMAND]

FLAGS:
    -C, --clean      Produces clean code output, without block label comments.
    -F, --force      Forces building, although it would result in overwriting changed files.
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -o, --code <path>          Directory to output code files to. Optional. Defaults to 'path -> code' from config file.
    -c, --config <path>        Sets the config file path [default: Yarner.toml]
    -d, --docs <path>          Directory to output documentation files to. Optional. Defaults to 'path -> docs' from
                               config file.
    -e, --entrypoint <name>    The named entrypoint to use when tangling code. Optional. Defaults to 'path ->
                               entrypoint', or to the unnamed code block(s).
    -r, --root <path>          Root directory. Optional. Defaults to 'path -> root' from config file, or to the current
                               directory.

ARGS:
    <FILES>...    The input source file(s) as glob pattern(s). Optional. Defaults to 'path -> files' from config
                  file.

SUBCOMMANDS:
    help       Prints this message or the help of the given subcommand(s)
    init       Creates a yarner project in the current directory
    reverse    Reverse mode: play back code changes into source files
    watch      Watch files and build project on changes
```
