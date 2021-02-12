# Command line arguments

Some configurations can be overwritten using command line arguments.

[[_TOC_]]

## CLI help

To get help on command line options, use `yarner -h`.

```plaintext
Literate programming compiler
  https://github.com/mlange-42/yarner

The normal workflow is:
 1) Create a project with
    > yarner init
 2) Process the project by running
    > yarner

USAGE:
    yarner [FLAGS] [OPTIONS] [input]... [SUBCOMMAND]

FLAGS:
    -C, --clean      Produces clean code output, without block label comments.
    -F, --force      Forces building, although it would result in overwriting changed files.
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -o, --output <code_dir>          Output tangled code files to this directory. If none is specified, uses 'path' ->
                                     'code' from config file.
    -c, --config <config>            Sets the config file name [default: Yarner.toml]
    -d, --docs <doc_dir>             Directory to output weaved documentation files to. If none is specified, uses
                                     'path' -> 'docs' from config file.
    -e, --entrypoint <entrypoint>    The named entrypoint to use when tangling code. Defaults to the unnamed code block.
    -l, --language <language>        The language to output the tangled code in. Only code blocks in this language will
                                     be used.
    -r, --root <root>                Root directory. If none is specified, uses 'path' -> 'root' from config file.
                                     Default: current directory.

ARGS:
    <input>...    The input source file(s) as glob pattern(s). If none are specified, uses 'path' -> 'files' from
                  config file.

SUBCOMMANDS:
    help       Prints this message or the help of the given subcommand(s)
    init       Creates a yarner project in the current directory
    reverse    Reverse mode: play back code changes into source files
```
