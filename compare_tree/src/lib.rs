use std::error::Error;

pub fn run(configuration: &Config) -> Result<(), Box<dyn Error>> {
    println!("Reference path {}", configuration.reference_path);
    println!("Other path {}", configuration.other_path);
    Ok(())
}

pub struct Config {
    reference_path: String,
    other_path: String
}

impl Config {
    pub fn build(args: &[String]) -> Result<Config, & 'static str> {
        if args.len() < 3 {
            return Err("Waiting for 2 arguments not {args.len()}");
        }
        let reference_path = args[1].clone();
        let other_path = args[2].clone();
        Ok(Config {reference_path, other_path})
    }
}
