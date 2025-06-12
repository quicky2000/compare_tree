use std::env::args;
use std::process;
use compare_tree::Config;

fn main() {
    println!("Hello, world!");
    let args: Vec<String> = args().collect();
    dbg!(&args);


    let configuration = Config::build(&args).unwrap_or_else(|err| {
        eprintln!("Error when parsing aguments : {err}");
        process::exit(-1);
    });

    if let Err(error) = compare_tree::run(&configuration) {
        eprintln!("Error occurr {error}");
        process::exit(-1);
    }
}

