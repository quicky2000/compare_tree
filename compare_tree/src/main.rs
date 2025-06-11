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

    let configuration = Config::new(&args);
    println!("Reference path {}", configuration.reference_path);
    println!("Other path {}", configuration.other_path);
}

struct Config {
    reference_path: String,
    other_path: String
}

impl Config {
    fn new(args: &[String]) -> Config {
        let reference_path = args[1].clone();
        let other_path = args[2].clone();
        Config {reference_path, other_path}
    }
}
