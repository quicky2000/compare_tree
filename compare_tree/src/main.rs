use std::env::args;
use std::process;

fn main() {
    println!("Hello, world!");
    let args: Vec<String> = args().collect();
    dbg!(&args);

    if args.len() < 3 {
        println!("Waiting for 2 arguments");
        process::exit(-1);
    }
}
