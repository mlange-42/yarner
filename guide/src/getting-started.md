# Getting started

Yarner is very easy to use from the command line, requiring only two different commands, `yarner init` and `yarner`. This chapter demonstrates how to set up a project and extract sources from Markdown.

[[_TOC_]]

## Create a project

To set up a new project, use the `init` sub-command. Run the following in your project's base directory:

```plaintext
> yarner init
```

This creates a file `Yarner.toml` with default settings, and a file `README.md` as starting point for Literate Programming.

The generated file already contains some content to get started with Yarner's basic features. These are explained in detail in the subsequent chapters.

## Build a project

To build the project (i.e. extract code and create documentation), simply run:

```plaintext
> yarner
```

This creates two sub-directories, one containing the extracted code (a minimal but working Rust project), and another containing the final documentation.

Note that the contents of these directories can then be treated as usual, i.e. compiling the code with the normal compiler, or rendering Markdown to HTML or PDF.

## What's next

The following chapters explain the use of Yarner in detail, feature by feature.

To get an impression how Yarner projects look like, or to find a template to get started with your preferred programming language, see also the [examples](https://github.com/mlange-42/yarner/tree/master/examples) in the [GitHub repository](https://github.com/mlange-42/yarner).
