# Writing plugins

[[_TOC_]]

## Basic workflow

Each plugin is called by Yarner during the build process. It receives all documents in their parsed state, after transclusions are performed and code is extracted, but before documentation is printed out. Each plugin should report back with the changed documents, and potentially added documents.

## Rust library

To use the [Rust](https://rust-lang.org) crate `yarner-lib` to write plugins, add it to the dependencies of your `Cargo.toml`:

```toml
[package]
...

[dependencies]
yarner-lib = "0.5"
...
```

Besides the `struct`s that make up a document, the library offer some convenience functions for JSON conversion. Here is an example plugin that adds a simple text paragraph to the end of each document:

```rust
use yarner_lib::*;

fn main() {
    std::process::exit(match run() {
        Ok(()) => 0,
        Err(err) => {
            eprintln!("ERROR: {}", err);
            1
        }
    });
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    // Get documents from stdin JSON
    let mut data = yarner_lib::parse_input()?;

    // Get options from the plugin's section in the Yarner.toml
    let _foo = data.context.config.get("foo").unwrap();

    // Manipulate documents
    for (_path, doc) in data.documents.iter_mut() {
        doc.nodes.push(Node::Text(TextBlock {
            text: vec![
                String::new(),
                String::from("Edited by an example plugin."),
            ],
        }));
    }

    // Convert documents back to JSON and print them to stdout
    yarner_lib::write_output(&data)?;
    Ok(())
}
```

See the [Known plugins](./known.md) for more complex code examples.

## JSON schema

For plugins in languages other than Rust, a JSON schema is provided in the [GitHub repository](https://github.com/mlange-42/yarner), folder [schemas](https://github.com/mlange-42/yarner/tree/master/schemas). [`yarner-data.json`](https://github.com/mlange-42/yarner/tree/master/schemas/yarner-data.json) describes the data passed from Yarner to plugins (context with config, as well as documents), and back to Yarner.
