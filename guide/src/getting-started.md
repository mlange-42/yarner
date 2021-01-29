# Getting started

Yarner is very easy to use from the command line, requiring only two different commands, `yarner create` and `yarner`. This chapter demonstrates how to set up a project and extract sources from Marksdown.

[[_TOC_]]

## Create a project

To set up a new project, use the `create` sub-command. Run the following in your project's base directory:

```plaintext
> yarner create README.md
```

This creates a file `Yarner.toml` with default settings, and a file `README.md.md` as starting point for Literate Programming (don't care for the double extension for now).

The generated Markdown file already contains some content to get started with Yarner's basic features. These are explained in detail in the subsequent chapters.

## Build a project

To build/"compile" the project (extract code and create documentation), simply run:

```plaintext
> yarner
```

This creates two sub-directories, one containing the extracted code, and the other containing the final documentation.

Note that the contents of these directories can then be treated as usual, i.e. compiling the code with the normal compiler, or rendering Markdown to HTML or PDF.
