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

    let reference_path = &args[1];
    let other_path = &args[2];
    println!("Reference path {reference_path}");
    println!("Other path {other_path}");
}
