use std::env::args;

fn main() {
    println!("Hello, world!");
    let args: Vec<String> = args().collect();
    dbg!(args);
}
