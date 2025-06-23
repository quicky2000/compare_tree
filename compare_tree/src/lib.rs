use std::error::Error;

mod sha1;

pub fn run(configuration: &Config) -> Result<(), Box<dyn Error>> {
    println!("Reference path {}", configuration.reference_path);
    println!("Other path {}", configuration.other_path);
    //let data = Vec::<u8>::new();
    //let data = vec!('a' as u8);
    let data: Vec<u8> = Vec::from("Hello world");
    let key = sha1::compute_sha1(data);
    println!("Key is {key}");
    Ok(())
}

#[derive(PartialEq)]
#[derive(Debug)]
pub struct Config {
    reference_path: String,
    other_path: String
}

impl Config {
    pub fn build(mut args: impl Iterator <Item = String>) -> Result<Config, & 'static str> {
        // Ignore command name
        args.next();
        let reference_path = match args.next() {
            Some(value) => value,
            None => return Err("No reference path provided")
        };
        let other_path = match args.next() {
            Some(value) => value,
            None => return Err("No other path provided")
        };
        Ok(Config {reference_path, other_path})
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse() {
        let args: [String; 3] = [String::from("command"), String::from("reference"), String::from("other")];
        let ref_config = Config {
            reference_path: "reference".to_string(),
            other_path: "other".to_string()
        };
        let result = Config::build(args.into_iter()).unwrap();
        assert_eq!(ref_config, result);
    }
    #[test]
    fn test_parse_fail() {
        let args = vec!["reference".to_string(), "other".to_string()];
        assert!(Config::build(args.into_iter()).is_err());
    }

}

