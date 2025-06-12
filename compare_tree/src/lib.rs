use std::error::Error;

pub fn run(configuration: &Config) -> Result<(), Box<dyn Error>> {
    println!("Reference path {}", configuration.reference_path);
    println!("Other path {}", configuration.other_path);
    Ok(())
}

#[derive(PartialEq)]
#[derive(Debug)]
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse() {
        let args = ["command".to_string(), "reference".to_string(), "other".to_string()];
        let ref_config = Config {
            reference_path: "reference".to_string(),
            other_path: "other".to_string()
        };
        let result = Config::build(&args).unwrap();
        assert_eq!(ref_config, result);
    }
    #[test]
    fn test_parse_fail() {
        let args = ["reference".to_string(), "other".to_string()];
        assert!(Config::build(&args).is_err());
    }
}
