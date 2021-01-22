fn main() {
    start();
    for i in 0..10 {
        step(i);
    }
    stop();
}
fn start() {
    println!("Starting");
}
fn stop() {
    println!("Stopping");
}
fn step(step: i32) {
    println!("Step {}", step);
}
