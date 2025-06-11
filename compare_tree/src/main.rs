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

    let (reference_path, other_path) = parse_arguments(&args);
    println!("Reference path {reference_path}");
    println!("Other path {other_path}");
}

fn parse_arguments(args: &[String]) -> (&String, &String) {
    let reference_path = &args[1];
    let other_path = &args[2];
    (reference_path, other_path)
}
