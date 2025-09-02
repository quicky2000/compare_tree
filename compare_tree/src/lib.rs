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

fn analyse_filetree(path: PathBuf) -> Result<filetree_info::FileTreeInfo, String> {
        let string_path = path.to_str().ok_or(format!("to_str() issue with {}", path.display()))?;

        // Get iterator to list directory content
        let dir_iter_result = fs::read_dir(string_path);
        let dir_iter = match dir_iter_result {
            Ok(dir_iter) => dir_iter,
            Err(_e) => return Err(format!("problem with dir_iter on {}", string_path).into())
        };
        let mut nb_item: u32 = 0;
        let mut height: u32 = 0;
        let mut keys = Vec::new();

        // List directory content
        for item_result in dir_iter {

            let item = match item_result {
                Ok(item) => item,
                Err(_e) => return Err(format!("Issue with item").into())
            };

            let item_path = item.path();
            println!("Analyse => {}", item_path.display());
            let item_path_str = item_path.to_str().ok_or(format!("to_str() issue with {}", item.path().display()))?;

            // Get item metadata
            let metadata_result = item.metadata();
            if metadata_result.is_err() {
                return Err(format!("Unable to collect metadata from file {}", item_path_str).into());
            }
            let metadata = metadata_result.unwrap();

            // Treat items depending on its type
            if metadata.is_dir() {
                println!("{} is a directory", item_path_str);
                let filetree_info = analyse_filetree(item.path())?;
                nb_item += filetree_info.nb_item;
                // Ignore empty directories
                if filetree_info.nb_item != 0 {
                    keys.push(filetree_info.sha1);
                    if height < filetree_info.height + 1 {
                        height = filetree_info.height + 1;
                    }
                }
            }
            if metadata.is_file() {
                println!("{} is a file", item_path_str);
                keys.push(compute_file_sha1(item_path_str)?);
                nb_item += 1;
            }
            if metadata.is_symlink() {
                println!("{} is a link", item_path_str);
                keys.push(compute_link_sha1(item_path_str)?);
                nb_item += 1;
            }
        }
        println!("Analyse => {} items at this level", nb_item);
        // Sort SHA1 keys to be independant of directory listing order
        keys.sort();
        keys.iter().for_each(|x| println!("{x:?}"));

        // Converts all sha1 + number of items to byte in order to compute SHA1 of this directory
        let mut data = Vec::<u8>::new();
        keys.iter().for_each(|k|data.extend(k.to_bytes()));
        data.extend(nb_item.to_le_bytes());

        Ok(filetree_info::FileTreeInfo{name: string_path.into(),
                                       height: height,
                                       sha1: sha1::compute_sha1(data),
                                       nb_item: nb_item})
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
        sha1: sha1::compute_sha1(vec!(0)),
        nb_item: 13
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
    let analyse_result = analyse_filetree(to_analyse);
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

    fn analyse_empty_dir(name: &str) -> filetree_info::FileTreeInfo {
        let create_result = fs::create_dir(name);
        assert!(create_result.is_ok());
        let mut to_analyse = PathBuf::new();
        to_analyse.push(name);
        let analyse_result = analyse_filetree(to_analyse);
        let rm_result = fs::remove_dir(name);
        assert!(rm_result.is_ok());
        assert!(analyse_result.is_ok());
        analyse_result.unwrap()
    }

    #[test]
    fn test_check_analyse_empty_dir() {
        let my_info = filetree_info::FileTreeInfo {
            name: "empty".to_string(),
            height: 0,
            sha1: sha1::compute_sha1(vec!(0,0,0,0)),
            nb_item: 0
        };
        assert_eq!(my_info, analyse_empty_dir("empty"));
    }

    #[test]
    fn test_check_analyse_empty_dir2() {
        assert!(filetree_info::equivalent(&analyse_empty_dir("empty1"), &analyse_empty_dir("empty2")));
    }

    use std::fs::File;
    use std::io::prelude::*;

    #[test]
    fn test_compare_different_names() {
        {
            assert!(fs::create_dir("reference").is_ok());
            let mut file1 = File::create("reference/file1.txt").expect("Unable to create file1");
            assert!(file1.write_all(b"Hello world!").is_ok());
            assert!(fs::create_dir("other").is_ok());
            let mut file2 = File::create("other/file2.txt").expect("Unable to create file2");
            assert!(file2.write_all(b"Hello world!").is_ok());
            assert!(fs::create_dir("other/empty").is_ok());
        }
        let mut path1 = PathBuf::new();
        path1.push("reference");
        let mut path2 = PathBuf::new();
        path2.push("other");
        assert!(filetree_info::equivalent(&analyse_filetree(path1).expect("Error with reference"), &analyse_filetree(path2).expect("Error with other")));
        assert!(fs::remove_dir_all("reference").is_ok());
        assert!(fs::remove_dir_all("other").is_ok());
    }

    #[test]
    fn test_height() {
        {
            assert!(fs::create_dir_all("reference2/level1/level2").is_ok());
            let mut file1 = File::create("reference2/level1/level2/file1.txt").expect("Unable to create file1");
            assert!(file1.write_all(b"Hello world!").is_ok());
            assert!(fs::create_dir("reference2/level1_bis").is_ok());
            let mut file2 = File::create("reference2/level1_bis/file2.txt").expect("Unable to create file2");
            assert!(file2.write_all(b"Hello world!").is_ok());
            let mut file3 = File::create("reference2/file3.txt").expect("Unable to create file3");
            assert!(file3.write_all(b"Hello world!").is_ok());
        }
        let mut path1 = PathBuf::new();
        path1.push("reference2");
        assert_eq!(2, analyse_filetree(path1).expect("Error with reference").height);
        assert!(fs::remove_dir_all("reference2").is_ok());
    }
}

