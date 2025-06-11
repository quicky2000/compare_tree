use std::env::args;
use std::process;
use std::error::Error;

fn main() {
    println!("Hello, world!");
    let args: Vec<String> = args().collect();
    dbg!(&args);


    let configuration = Config::build(&args).unwrap_or_else(|err| {
        println!("Error when parsing aguments : {err}");
        process::exit(-1);
    });

    if let Err(error) = run(&configuration) {
        println!("Error occurr {error}");
        process::exit(-1);
    }
}

fn run(configuration: &Config) -> Result<(), Box<dyn Error>> {
    println!("Reference path {}", configuration.reference_path);
    println!("Other path {}", configuration.other_path);
    Ok(())
}

struct Config {
    reference_path: String,
    other_path: String
}

impl Config {
    fn build(args: &[String]) -> Result<Config, & 'static str> {
        if args.len() < 3 {
            return Err("Waiting for 2 arguments not {args.len()}");
        }
        let reference_path = args[1].clone();
        let other_path = args[2].clone();
        Ok(Config {reference_path, other_path})
    }
}
