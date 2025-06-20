use std::fmt;
use std::error::Error;

pub fn run(configuration: &Config) -> Result<(), Box<dyn Error>> {
    println!("Reference path {}", configuration.reference_path);
    println!("Other path {}", configuration.other_path);
    Ok(())
}

#[derive(PartialEq)]
#[derive(Debug)]
struct Sha1Key {
    words: [u32; 5]
}

impl std::fmt::Display for Sha1Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.words.iter().map(|x| write!(f, "{:08X}", x)).rev().collect()
    }
}

impl Sha1Key {
    fn from_array(words: [u32; 5]) -> Sha1Key {
        Sha1Key {words: words}
    }
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

    impl Sha1Key {
        fn new(word0: u32, word1: u32, word2: u32, word3: u32, word4: u32) -> Sha1Key {
            Sha1Key {words: [word0, word1, word2, word3, word4]}
        }
    }

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

    #[test]
    fn test_sha1_key_compare() {
        let key_ref = Sha1Key::new(0x0, 0x1, 0x2, 0x3, 0x4);
        let key_other = Sha1Key::new(0x0, 0x1, 0x2, 0x3, 0x5);
        assert_ne!(key_ref, key_other);
    }
    #[test]
    fn test_sha1_key_display() {
        let key_ref = Sha1Key::new(0x0, 0x1, 0x2, 0x3, 0x4);
        assert_eq!(format!("{}", key_ref), "0000000400000003000000020000000100000000");
    }
    #[test]
    fn test_sha1_key_from_array() {
        let key_ref = Sha1Key::new(0x0, 0x1, 0x2, 0x3, 0x4);
        let key_array = Sha1Key::from_array([0x0, 0x1, 0x2, 0x3, 0x4]);
        assert_eq!(key_ref, key_array);
    }
}
