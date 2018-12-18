mod hello;
mod world;

pub fn hello_world() -> String {
    format!("{} {}", hello::hello(), world::world())
}
