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

    let check = fs::exists(&configuration.other_path);
    if check.is_err() || !check.unwrap() {
        return Err(format!("file {} do not exist", configuration.other_path).into())
    }
    let metadata_result = fs::symlink_metadata(&configuration.other_path);
    if metadata_result.is_err() {
        return Err(format!("Unable to collect metadata from file {}", configuration.other_path).into());
    }
    let metadata = metadata_result.unwrap();
    if metadata.is_dir() {
        println!("{} is a directory", configuration.other_path);
        let dir_iter_result = fs::read_dir(&configuration.other_path);
        let dir_iter = match dir_iter_result {
            Ok(dir_iter) => dir_iter,
            Err(_e) => return Err(format!("problem with dir_iter on {}", configuration.other_path).into())
        };
        for item_result in dir_iter {
            let item = match item_result {
                Ok(item) => item,
                Err(_e) => return Err(format!("Issue with item").into())
            };
            println!("=> {}", item.path().display());
        }

    }
    if metadata.is_file() {
        println!("{} is a file", configuration.other_path)
    }
    if metadata.is_symlink() {
        println!("{} is a link", configuration.other_path)
    }
    let key_result = compute_link_sha1(&configuration.other_path);
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

fn compute_link_sha1(link_name: &str) -> Result<sha1::Sha1Key, String> {
    let check = fs::exists(link_name);
    if check.is_err() || !check.unwrap() {
        return Err(format!("file {link_name} do not exist").to_string())
    }
    let read_result = fs::read_link(link_name);
    let path = match read_result {
        Ok(p) => p,
        Err(_e) => return Err(format!("Fail to read path of link {link_name}").to_string())
    };
    let path_str = match path.to_str() {
        Some(str) => str,
        None => return Err(format!("Fail to convert link {link_name} path to string").to_string())
    };
    let data: Vec<u8> = Vec::from(path_str);
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

