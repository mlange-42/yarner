# Hello world

This is a very complicated hello world example:

```rs
mod hello;
mod world;
mod hello_world;

fn main() {
  println!("{} {}", hello::hello(), world::world());
  println!("{}", hello_world::hello_world());
}
```
