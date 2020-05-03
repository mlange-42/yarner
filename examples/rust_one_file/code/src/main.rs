fn main() {
    start();
    for _i in 0..10 {
        step();
    }
    stop();
}

fn start() {
    println!("start");
}
fn step() {
    println!("    step");
}
fn stop() {
    println!("stop");
}