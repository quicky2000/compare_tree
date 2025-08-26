/*    This file is part of compare_tree
      Copyright (C) 2025  Julien Thevenon ( julien_thevenon at yahoo.fr )

      This program is free software: you can redistribute it and/or modify
      it under the terms of the GNU General Public License as published by
      the Free Software Foundation, either version 3 of the License, or
      (at your option) any later version.

      This program is distributed in the hope that it will be useful,
      but WITHOUT ANY WARRANTY; without even the implied warranty of
      MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
      GNU General Public License for more details.

      You should have received a copy of the GNU General Public License
      along with this program.  If not, see <http://www.gnu.org/licenses/>
*/
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

mod sha1;
mod filetree_info;

fn analyse_filetree(path: PathBuf, height: u32) -> Result<filetree_info::FileTreeInfo, String> {
        let string_path = path.to_str().unwrap();
        let dir_iter_result = fs::read_dir(string_path);
        let dir_iter = match dir_iter_result {
            Ok(dir_iter) => dir_iter,
            Err(_e) => return Err(format!("problem with dir_iter on {}", string_path).into())
        };
        let mut nb_item: u32 = 0;
        let mut data = Vec::<u8>::new();
        for item_result in dir_iter {
            let item = match item_result {
                Ok(item) => item,
                Err(_e) => return Err(format!("Issue with item").into())
            };
            nb_item += 1;
            println!("Analyse => {}", item.path().display());
        }
        println!("Analyse => {} items at this level", nb_item);
        data.extend(nb_item.to_le_bytes());
        Ok(filetree_info::FileTreeInfo{name: string_path.into(),
                                       height: height,
                                       sha1: sha1::compute_sha1(data)})
}

fn check_directory(name: &str ) -> Result<bool, String> {

    let check = fs::exists(name);
    if check.is_err() || !check.unwrap() {
        return Err(format!("file {} do not exist", name).into())
    }

    let metadata_result = fs::symlink_metadata(name);
    if metadata_result.is_err() {
        return Err(format!("Unable to collect metadata from file {}", name).into());
    }
    let metadata = metadata_result.unwrap();
    if !metadata.is_dir() {
        return Err(format!("{} is not a directory", name).into());
    }

    Ok(true)
}

pub fn run(configuration: &Config) -> Result<(), Box<dyn Error>> {
    println!("Reference path {}", configuration.reference_path);
    println!("Other path {}", configuration.other_path);

    let my_info = filetree_info::FileTreeInfo {
        name: "my_filetree".to_string(),
        height: 8,
        sha1: sha1::compute_sha1(vec!(0))
    };
    println!("{}", my_info);

    let result = check_directory(&configuration.reference_path);
    if result.is_err() {
        return Err(result.err().unwrap().into());
    }
    let result = check_directory(&configuration.other_path);
    if result.is_err() {
        return Err(result.err().unwrap().into());
    }

    let mut to_analyse = PathBuf::new();
    to_analyse.push(&configuration.reference_path);
    let analyse_result = analyse_filetree(to_analyse, 1);
    let analyse = match analyse_result {
        Ok(k) => k,
        Err(e) => return Err(e.into())
    };
    println!("{}", analyse);

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
    #[test]
    fn test_check_directory() {
        assert!(check_directory("target").unwrap());
    }
    #[test]
    fn test_check_directory_fail() {
        assert!(check_directory("dummy_target").is_err());
    }
   #[test]
    fn test_check_analyse_empty_dir() {
        let create_result = fs::create_dir("empty");
        assert!(create_result.is_ok());
        let my_info = filetree_info::FileTreeInfo {
            name: "empty".to_string(),
            height: 1,
            sha1: sha1::compute_sha1(vec!(0,0,0,0))
        };
        let mut to_analyse = PathBuf::new();
        to_analyse.push("empty");
        let analyse_result = analyse_filetree(to_analyse, 1);
        assert!(analyse_result.is_ok());
        let analyse = analyse_result.unwrap();
        assert_eq!(my_info, analyse);
        let rm_result = fs::remove_dir("empty");
        assert!(rm_result.is_ok());
    }

}

