// <@README.md.md#file:src/main.rs
use rust_md;

fn main() {
    start();
    for _i in 0..10 {
        step();
    }
    stop();
    rust_md::secret();
}

// <@README.md.md#Code of the functions
fn start() {
    println!("start");
}
// @>README.md.md#Code of the functions
// <@README.md.md#Code of the functions
fn step() {
    println!("    step");
}
// @>README.md.md#Code of the functions
// <@README.md.md#Code of the functions
fn stop() {
    println!("stop");
}
// @>README.md.md#Code of the functions
// @>README.md.md#file:src/main.rs
