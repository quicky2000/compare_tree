use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::Read;

mod sha1;

pub fn run(configuration: &Config) -> Result<(), Box<dyn Error>> {
    println!("Reference path {}", configuration.reference_path);
    println!("Other path {}", configuration.other_path);
    let key_result = compute_file_sha1(&configuration.reference_path);
    let key = match key_result {
        Ok(k) => k,
        Err(e) => return Err(e.into())
    };
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

fn compute_file_sha1(file_name: &str) -> Result<sha1::Sha1Key, String> {
    let check = fs::exists(file_name);
    if check.is_err() || !check.unwrap() {
        return Err(format!("file {file_name} do not exist").to_string())
    }

    let file_result = File::open(file_name);
    let mut file = match file_result {
        Ok(f) => f,
        Err(_e) => return Err(format!("Unable to open file {}", file_name))
    };

    let mut data: Vec<u8> = Vec::new();
    let read_result = file.read_to_end(&mut data);
    if read_result.is_err() {
        return Err(format!("Unable to read content of file {file_name}").to_string());
    }
    println!("{:?}", data);
    Ok(sha1::compute_sha1(data))
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

