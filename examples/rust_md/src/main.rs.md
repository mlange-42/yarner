# Main function

Everything starts in the `main` function:

```rust
fn main() {
    start();
    for _i in 0..10 {
        step();
    }
    stop();
}

// ==> Code of the functions.
```

The code for the `start()` function looks like this:

```rust - Code of the functions
fn start() {
    println!("start");
}
```

The code for the `step()` function looks like this:

```rust - Code of the functions
fn step() {
    println!("    step");
}
```

And finally, the code for the `stop()` function looks like this:

```rust - Code of the functions
fn stop() {
    println!("stop");
}
```

Back to [README](../README.md.md)
