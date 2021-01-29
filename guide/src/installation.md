# Installation

There are multiple ways to install Yarner:

[[_TOC_]]

## Binaries

1. Download the [latest binaries](https://github.com/mlange-42/yarner/releases) for your platform  
   (Binaries are available for Linux, Windows and macOS)
2. Unzip somewhere
3. *Optional:* add directory `yarner` to your `PATH` environmental variable

## From GitHub using `cargo`

In case you have [Rust](https://www.rust-lang.org/) installed, you can install with `cargo`:

```
cargo install --git https://github.com/mlange-42/yarner yarner
```

## Clone and build

To build Yarner locally, e.g. to contribute to the project, you will have to clone the repository on your local machine:

```
git clone https://github.com/mlange-42/yarner
```

`cd` into `yarner/` and run

```
cargo build
```

The resulting binary can be found in `yarner/target/debug/` under the name `yarner` or `yarner.exe`.
