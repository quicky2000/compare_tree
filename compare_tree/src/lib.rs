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
use std::io::BufWriter;
use std::io::BufReader;
use std::io;
use std::path::PathBuf;
use std::io::prelude::*;


mod sha1;
mod filetree_info;

fn analyse_filetree(path: PathBuf, output: &mut impl Write) -> Result<filetree_info::FileTreeInfo, String> {
    let string_path = path.to_str().ok_or(format!("to_str() issue with {}", path.display()))?;

    // Get iterator to list directory content
    let dir_iter_result = fs::read_dir(string_path);
    let dir_iter = match dir_iter_result {
        Ok(dir_iter) => dir_iter,
        Err(_e) => return Err(format!("problem with dir_iter on {}", string_path).into())
    };
    let mut nb_item: u32 = 0;
    let mut height: u32 = 1;
    let mut keys = Vec::new();

    // List directory content
    for item_result in dir_iter {

        let item = match item_result {
            Ok(item) => item,
            Err(_e) => return Err(format!("Issue with item").into())
        };

        let item_path = item.path();
        if cfg!(test) { println!("Analyse => {}", item_path.display()); }
        let item_path_str = item_path.to_str().ok_or(format!("to_str() issue with {}", item.path().display()))?;

        // Get item metadata
        let metadata_result = item.metadata();
        if metadata_result.is_err() {
            return Err(format!("Unable to collect metadata from file {}", item_path_str).into());
        }
        let metadata = metadata_result.unwrap();

        // Treat items depending on its type
        if metadata.is_dir() {
            if cfg!(test) { println!("{} is a directory", item_path_str); }
            let filetree_info = analyse_filetree(item.path(), output)?;
            nb_item += filetree_info.nb_item;
            // Ignore empty directories
            if filetree_info.nb_item != 0 {
                keys.push(filetree_info.sha1);
                if height < filetree_info.height + 1 {
                    height = filetree_info.height + 1;
                }
            }
        }
        if metadata.is_file() || metadata.is_symlink() {
            let sha1 = if metadata.is_file() {
                if cfg!(test) { println!("{} is a file", item_path_str); }
                compute_file_sha1(item_path_str)?
            } else {
                if cfg!(test) { println!("{} is a link", item_path_str); }
                compute_link_sha1(item_path_str)?

            };
            let result = filetree_info::FileTreeInfo{name: String::from(item_path_str),
                                                     height: 0,
                                                     sha1: sha1.clone(),
                                                     nb_item: 0};
            let write_result = output.write(format!("{}\n", result).as_bytes());
            if write_result.is_err() {
                return Err(format!("Unable to write result of {}", item_path_str).into());
            }
            keys.push(sha1);
            nb_item += 1;
        }
    }
    if cfg!(test) { println!("Analyse => {} items at this level", nb_item); }
    // Sort SHA1 keys to be independant of directory listing order
    keys.sort();
    //keys.iter().for_each(|x| println!("{x:?}"));

    // Converts all sha1 + number of items to byte in order to compute SHA1 of this directory
    let mut data = Vec::<u8>::new();
    keys.iter().for_each(|k|data.extend(k.to_bytes()));
    data.extend(nb_item.to_le_bytes());

    let result = filetree_info::FileTreeInfo{name: string_path.into(),
                                             height: height,
                                             sha1: sha1::compute_sha1(data),
                                             nb_item: nb_item};
    let write_result = output.write(format!("{}\n", result).as_bytes());
    if write_result.is_err() {
        return Err(format!("Unable to write result of {}", string_path).into());
    }

    Ok(result)
}

fn dump_name(name: & str) -> String {
    let mut filename = String::from(name);
    filename.push_str("_dump.txt");
    return filename;
}

fn dump_dir(name: & str) -> String {
    let mut filename = String::from(name);
    filename.push_str("_dumps");
    return filename;
}

fn split_name(name: &str, height: u32) -> String {
    let mut filename = dump_dir(name);
    filename.push('/');
    filename.push_str(&height.to_string());
    filename.push_str(".txt");
    return filename;
}

fn analyse(name: &str) -> Result<filetree_info::FileTreeInfo, String> {
    let filename = dump_name(name);
    let file = File::create(&filename).expect(format!("Unable to create file {}", filename).as_str());
    let mut buf = BufWriter::new(file);
    let mut path = PathBuf::new();
    path.push(name);
    analyse_filetree(path, &mut buf)
}

fn generate_dump(name: &str) -> Result<u32, String> {
    let result: u32;
    let check = fs::exists(dump_name(name));
    if check.is_ok() && check.unwrap() {
        println!("==> Parse existing dump for {}", name);
        let file_result = File::open(dump_name(name));
        let file = match file_result {
            Ok(f) => f,
            Err(e) => return Err(format!("Unable to open file {} {}", dump_name(name), e))
        };
        let reader = BufReader::new(file);
        let mut line = String::from("");
        for line_result in reader.lines() {
            if line_result.is_ok() {
                line = line_result.unwrap();
            }
            else {
                return Err(format!("Unable to read from {}", dump_name(name)));
            }
        }
        let filetree_info = filetree_info::FileTreeInfo::from(&line)?;
        result = filetree_info.height;
    }
    else {
        println!("==> Generate dump for {}", name);
        let analyse = analyse(name)?;
        result = analyse.height;
        let check = fs::exists(dump_dir(name));
        if check.is_err() {
                return Err(format!("Unable to determine if directory {} exists", dump_dir(name)));
        }
        if check.unwrap() {
            let rm_result = fs::remove_dir_all(dump_dir(name));
            if rm_result.is_err() {
                return Err(format!("Unable to clean directory {}", dump_dir(name)));
            }
        }
    }
    Ok(result)
}

fn generate_split(name: &str, height: u32) -> Result<(), String> {
        println!("==> Prepare split for '{}'", name);
        let check = fs::exists(dump_dir(name));
        if check.is_err() {
                return Err(format!("Unable to determine if directory {} exists", dump_dir(name)));
        }
        if !check.unwrap() {
            let rm_result = fs::create_dir(dump_dir(name));
            if rm_result.is_err() {
                return Err(format!("Unable to create directory {}", dump_dir(name)));
            }

            // Create a block to be sure writers are closed at the end
            {
                let mut files = Vec::new();

                // Create writers
                for i in 0..height + 1 {
                    let filename = split_name(name, i);
                    println!("===> Create split {filename}");
                    let file = File::create(&filename).expect(format!("Unable to create file {}", &filename).as_str());
                    files.push(BufWriter::new(file));
                }

                let file_result = File::open(dump_name(name));
                let file = match file_result {
                    Ok(f) => f,
                    Err(e) => return Err(format!("Unable to open file {} {}", dump_name(name), e))
                };
                // Populate files with content of dump
                let reader = BufReader::new(file);
                for line_result in reader.lines() {
                    let line = match line_result {
                        Ok(l) => l,
                        Err(e) => return Err(format!("Unable to read from {} : {}", dump_name(name), e))
                    };
                    let filetree_info = filetree_info::FileTreeInfo::from(&line)?;
                    assert!((filetree_info.height as usize) < files.len());
                    let write_result = files[filetree_info.height as usize].write(format!("{}\n", filetree_info).as_bytes());
                    if write_result.is_err() {
                        return Err(format!("Unable to write {} in {}", filetree_info, split_name(name, filetree_info.height)));
                    }
                }
            }
            // Sort splitted dumps
            for i in 0..height + 1 {
                let filename = split_name(name, i);
                println!("===> Sort split {filename}");
                let mut fileinfos = Vec::new();
                {
                    let file_result = File::open(&filename);
                    let file = match file_result {
                        Ok(f) => f,
                        Err(e) => return Err(format!("Unable to open file {} {}", dump_name(name), e))
                    };
                    let reader = BufReader::new(file);
                    for line_result in reader.lines() {
                        let line = match line_result {
                            Ok(l) => l,
                            Err(e) => return Err(format!("Unable to read from {} : {}", dump_name(name), e))
                        };
                        fileinfos.push(filetree_info::FileTreeInfo::from(&line)?);
                    }
                }
                fileinfos.sort();
                let mut path = PathBuf::new();
                path.push(&filename);
                let remove_result = fs::remove_file(path);
                if remove_result.is_err() {
                    return Err(format!("Unable to remove {} for sort", filename));
                }
                let file = File::create(&filename).expect(format!("Unable to create file {}", &filename).as_str());
                let mut writer = BufWriter::new(file);
                for item in fileinfos.iter() {
                    let write_result = writer.write(format!("{}\n", item).as_bytes());
                    if write_result.is_err() {
                        return Err(format!("Unable to write {} in {}", item, filename));
                    }
                }
            }
        }
        Ok(())
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

fn consume(io_iter: &mut io::Lines<io::BufReader<File>>) ->Result<String, String> {
    let result_iter = io_iter.next();
    if result_iter.is_none() {
        return Ok(String::from(""));
    }
    let result = match result_iter.unwrap() {
            Ok(s) => s,
            Err(e) => return Err(format!("Unable to read : {}", e))
        };
    return Ok(result);
}

fn compare_iter(mut reference: io::Lines<io::BufReader<File>> ,
                mut other: io::Lines<io::BufReader<File>>,
                to_remove: &mut Vec<(String, String)>) -> Result<(), String> {
    // File can never be empty as splitted files are created when encoutering a FileTreeInfo
    let mut ref_item = filetree_info::FileTreeInfo::from(&consume(&mut reference)?)?;
    let mut other_line = consume(&mut other)?;
    loop {
        // Check if we reach end of one of the files
        if other_line == "" {
            return Ok(())
        }
        let other_item = filetree_info::FileTreeInfo::from(&other_line)?;
        let other_len = other_item.name.chars().count();
        if to_remove.iter().any(|(_, x)| { x.chars().count () <= other_len && x == &(other_item.name.chars().take(x.chars().count()).collect::<String>())}) {
            other_line = consume(&mut other)?;
            continue;
        }
        if ref_item.equivalent(&other_item) {
            to_remove.push((ref_item.name.clone(), other_item.name));
            other_line = consume(&mut other)?;
        }
        else if ref_item.sha1 < other_item.sha1 {
            let ref_line = consume(&mut reference)?;
            if ref_line == "" {
                return Ok(());
            }
            ref_item = filetree_info::FileTreeInfo::from(&ref_line)?;
        }
        else {
            other_line = consume(&mut other)?;
        }
    }
}

fn compare(reference: &str, other: &str, height: u32) -> Result<Vec<(String, String)>, String> {
    println!("==> Analyse");
    let mut to_remove = Vec::new();
    for i in (0..height + 1).rev() {
        println!("===> Analyse height {}", i);
        let filename = split_name(reference, i);
        let file_result = File::open(&filename);
        let file = match file_result {
                Ok(f) => f,
                Err(e) => return Err(format!("Unable to open file {} {}", filename, e))
        };
        let reader_ref = BufReader::new(file);
        let filename = split_name(other, i);
        let file_result = File::open(&filename);
        let file = match file_result {
                Ok(f) => f,
                Err(e) => return Err(format!("Unable to open file {} {}", filename, e))
        };
        let reader_other = BufReader::new(file);
        compare_iter(reader_ref.lines(), reader_other.lines(), & mut to_remove)?;
    }
    Ok(to_remove)
}

fn compare_trees(reference: &str, other: &str) -> Result<Vec<(String, String)>, String> {
    let height_ref = generate_dump(reference)?;
    let height_other = generate_dump(other)?;
    println!("==> Dump result {} vs {}", height_ref, height_other);

    let common_height = if height_ref > height_other {height_other} else {height_ref};
    println!("===> Comparison will be done until height {}", common_height);

    generate_split(reference, height_ref)?;
    generate_split(other, height_other)?;

    compare(reference, other, common_height)
}

pub fn run(configuration: &Config) -> Result<(), Box<dyn Error>> {
    println!(" Reference path: '{}'", configuration.reference_path);
    println!("comparison path: '{}'", configuration.other_path);

    let result = check_directory(&configuration.reference_path);
    if result.is_err() {
        return Err(result.err().unwrap().into());
    }
    let result = check_directory(&configuration.other_path);
    if result.is_err() {
        return Err(result.err().unwrap().into());
    }

    let result = compare_trees(&configuration.reference_path, &configuration.other_path)?;

    println!("==> Results");
    result.iter().for_each(|(reference, other)| println!("{} TO REMOVE {}", reference, other));

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
    //println!("{:?}", data);
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
        let analyse_result = analyse(name);
        let rm_result = fs::remove_dir(name);
        assert!(rm_result.is_ok());
        assert!(analyse_result.is_ok());
        assert!(fs::remove_file(dump_name(name)).is_ok());
        analyse_result.unwrap()
    }

    #[test]
    fn test_check_analyse_empty_dir() {
        let my_info = filetree_info::FileTreeInfo {
            name: "empty".to_string(),
            height: 1,
            sha1: sha1::compute_sha1(vec!(0,0,0,0)),
            nb_item: 0
        };
        assert_eq!(my_info, analyse_empty_dir("empty"));
    }

    #[test]
    fn test_check_analyse_empty_dir2() {
        assert!(&analyse_empty_dir("empty1").equivalent(&analyse_empty_dir("empty2")));
    }

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
        assert!(&analyse("reference").expect("Error with reference").equivalent(&analyse("other").expect("Error with other")));
        assert!(fs::remove_dir_all("reference").is_ok());
        assert!(fs::remove_dir_all("other").is_ok());
        assert!(fs::remove_file(dump_name("reference")).is_ok());
        assert!(fs::remove_file(dump_name("other")).is_ok());
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
        assert_eq!(3, analyse("reference2").expect("Error with reference").height);
        assert!(fs::remove_dir_all("reference2").is_ok());
        assert!(fs::remove_file(dump_name("reference2")).is_ok());
    }
    #[test]
    fn test_compare_no_common1() {
        compare_generic("ref_dump1.txt", vec!(("toto".to_string(), "0000000400000003000000020000000100000000".to_string())),
                        "oth_dump1.txt", vec!(("tutu".to_string(), "0000000F00000003000000020000000100000000".to_string())),
                        vec!()
                       );
    }
    #[test]
    fn test_compare_common() {
        compare_generic("ref_dump2.txt", vec!(("toto".to_string(), "0000000400000003000000020000000100000000".to_string())),
                        "oth_dump2.txt", vec!(("tutu".to_string(), "0000000400000003000000020000000100000000".to_string())),
                        vec!(("toto".to_string(), "tutu".to_string()))
                       );
    }
    #[test]
    fn test_compare_false_common() {
        let ref_name = "ref_dump3.txt";
        let other_name = "other_dump3.txt";
        {
            let mut ref_dump = File::create(ref_name).expect("Unable to create ref dump");
            ref_dump.write(format!("{}\n",
                                   filetree_info::FileTreeInfo{name: "toto".to_string(),
                                                               height: 0,
                                                               nb_item: 0,
                                                               sha1: sha1::Sha1Key::from_string("0000000400000003000000020000000100000000").expect("From_string error")
                                                              }
                                   ).as_bytes()
                          ).expect("Error during write of ref dump");
            let mut other_dump = File::create(other_name).expect("Unable to create other dump");
            other_dump.write(format!("{}\n",
                                   filetree_info::FileTreeInfo{name: "tutu".to_string(),
                                                               height: 0,
                                                               nb_item: 2,
                                                               sha1: sha1::Sha1Key::from_string("0000000400000003000000020000000100000000").expect("From_string error")
                                                              }
                                   ).as_bytes()
                          ).expect("Error during write of other dump");
        }
        let ref_dump = File::open(ref_name).expect("Unable to open ref dump");
        let ref_bufreader = BufReader::new(ref_dump);
        let other_dump = File::open(other_name).expect("Unable to open other dump");
        let other_bufreader = BufReader::new(other_dump);
        let mut to_remove = Vec::<(String, String)>::new();
        compare_iter(ref_bufreader.lines(), other_bufreader.lines(), &mut to_remove).expect("Error during comparison");
        assert_eq!(Vec::<(String, String)>::new(), to_remove);
        assert!(fs::remove_file(ref_name).is_ok());
        assert!(fs::remove_file(other_name).is_ok());
    }
    fn create_dump(name: &str, list: Vec::<(String, String)>) {
            let mut dump = File::create(name).expect("Unable to create dump");
            list.iter().for_each(|(item_name, sha1)|{
                                 dump.write(format!("{}\n",
                                                    filetree_info::FileTreeInfo{name: item_name.to_string(),
                                                                                height: 0,
                                                                                nb_item: 0,
                                                                                sha1: sha1::Sha1Key::from_string(sha1).expect("From_string error")
                                                                               }
                                                   ).as_bytes()
                                           ).expect("Error during write of ref dump");}
                                );
    }
    fn compare_generic(ref_name: &str, ref_list: Vec::<(String, String)>,
                       oth_name: &str, oth_list: Vec::<(String, String)>,
                       ref_to_remove: Vec<(String, String)>
                      ) {
        create_dump(ref_name, ref_list);
        create_dump(oth_name, oth_list);
        let ref_dump = File::open(ref_name).expect("Unable to open ref dump");
        let oth_dump = File::open(oth_name).expect("Unable to open other dump");
        let ref_bufreader = BufReader::new(ref_dump);
        let oth_bufreader = BufReader::new(oth_dump);
        let mut to_remove = Vec::<(String, String)>::new();
        compare_iter(ref_bufreader.lines(), oth_bufreader.lines(), &mut to_remove).expect("Error during comparison");
        assert_eq!(ref_to_remove, to_remove);
        assert!(fs::remove_file(ref_name).is_ok());
        assert!(fs::remove_file(oth_name).is_ok());
    }
    #[test]
    fn test_compare_ref1() {
        compare_generic("ref_dump5.txt", vec!(("file_1".to_string(), "E57CC94793F1A408226B070046C2D6253E108C4A".to_string()),
                                              ("file_2".to_string(), "2CC944E46E5029A3AAFFE9554CD950C3C79694CC".to_string()),
                                              ("file_3".to_string(), "49584B5C027111E7A9F8F04BED3550A7FAA41DA4".to_string())
                                             ),
                        "oth_dump5.txt", vec!(("other1".to_string(), "E57CC94793F1A408226B070046C2D6253E108C4A".to_string())),
                        vec!(("file_1".to_string(),"other1".to_string()))
                       );
    }
    #[test]
    fn test_compare_ref2() {
        compare_generic("ref_dump6.txt", vec!(("file_1".to_string(), "E57CC94793F1A408226B070046C2D6253E108C4A".to_string()),
                                              ("file_2".to_string(), "2CC944E46E5029A3AAFFE9554CD950C3C79694CC".to_string()),
                                              ("file_3".to_string(), "49584B5C027111E7A9F8F04BED3550A7FAA41DA4".to_string())
                                             ),
                        "oth_dump6.txt", vec!(("other2".to_string(), "2CC944E46E5029A3AAFFE9554CD950C3C79694CC".to_string())),
                        vec!(("file_2".to_string(), "other2".to_string()))
                       );
    }
    #[test]
    fn test_compare_ref3() {
        compare_generic("ref_dump7.txt", vec!(("file_1".to_string(), "E57CC94793F1A408226B070046C2D6253E108C4A".to_string()),
                                              ("file_2".to_string(), "2CC944E46E5029A3AAFFE9554CD950C3C79694CC".to_string()),
                                              ("file_3".to_string(), "49584B5C027111E7A9F8F04BED3550A7FAA41DA4".to_string())
                                             ),
                        "oth_dump7.txt", vec!(("other3".to_string(), "49584B5C027111E7A9F8F04BED3550A7FAA41DA4".to_string())),
                        vec!(("file_3".to_string(), "other3".to_string()))
                       );
    }
    #[test]
    fn test_compare_other1() {
        compare_generic("ref_dump8.txt", vec!(("file_1".to_string(), "E57CC94793F1A408226B070046C2D6253E108C4A".to_string())),
                        "oth_dump8.txt", vec!(("other1".to_string(), "E57CC94793F1A408226B070046C2D6253E108C4A".to_string()),
                                              ("other2".to_string(), "2CC944E46E5029A3AAFFE9554CD950C3C79694CC".to_string()),
                                              ("other3".to_string(), "49584B5C027111E7A9F8F04BED3550A7FAA41DA4".to_string())),
                        vec!(("file_1".to_string(), "other1".to_string()))
                       );
    }
    #[test]
    fn test_compare_other2() {
        compare_generic("ref_dump9.txt", vec!(("file_1".to_string(), "2CC944E46E5029A3AAFFE9554CD950C3C79694CC".to_string())),
                        "oth_dump9.txt", vec!(("other1".to_string(), "E57CC94793F1A408226B070046C2D6253E108C4A".to_string()),
                                              ("other2".to_string(), "2CC944E46E5029A3AAFFE9554CD950C3C79694CC".to_string()),
                                              ("other3".to_string(), "49584B5C027111E7A9F8F04BED3550A7FAA41DA4".to_string())),
                        vec!(("file_1".to_string(), "other2".to_string()))
                       );
    }
    #[test]
    fn test_compare_other3() {
        compare_generic("ref_dump10.txt", vec!(("file_1".to_string(), "49584B5C027111E7A9F8F04BED3550A7FAA41DA4".to_string())),
                        "oth_dump10.txt", vec!(("other1".to_string(), "E57CC94793F1A408226B070046C2D6253E108C4A".to_string()),
                                               ("other2".to_string(), "2CC944E46E5029A3AAFFE9554CD950C3C79694CC".to_string()),
                                               ("other3".to_string(), "49584B5C027111E7A9F8F04BED3550A7FAA41DA4".to_string())),
                        vec!(("file_1".to_string(), "other3".to_string()))
                       );
    }
    #[test]
    fn test_compare_several_other1() {
        compare_generic("ref_dump11.txt", vec!(("file_1".to_string(), "49584B5C027111E7A9F8F04BED3550A7FAA41DA4".to_string())),
                        "oth_dump11.txt", vec!(("other1".to_string(), "E57CC94793F1A408226B070046C2D6253E108C4A".to_string()),
                                               ("other2".to_string(), "49584B5C027111E7A9F8F04BED3550A7FAA41DA4".to_string()),
                                               ("other3".to_string(), "49584B5C027111E7A9F8F04BED3550A7FAA41DA4".to_string())),
                        vec!(("file_1".to_string(), "other2".to_string()), ("file_1".to_string(), "other3".to_string()))
                       );
    }
    #[test]
    fn test_compare_several_other2() {
        compare_generic("ref_dump12.txt", vec!(("file_1".to_string(), "E57CC94793F1A408226B070046C2D6253E108C4A".to_string())),
                        "oth_dump12.txt", vec!(("other1".to_string(), "E57CC94793F1A408226B070046C2D6253E108C4A".to_string()),
                                               ("other2".to_string(), "E57CC94793F1A408226B070046C2D6253E108C4A".to_string()),
                                               ("other3".to_string(), "49584B5C027111E7A9F8F04BED3550A7FAA41DA4".to_string())),
                        vec!(("file_1".to_string(), "other1".to_string()), ("file_1".to_string(), "other2".to_string()))
                       );
    }
    #[test]
    fn test_compare_utf8() {
    compare_generic("ref_dump13.txt", vec!(("file_1".to_string(), "E57CC94793F1A408226B070046C2D6253E108C4A".to_string())),
                    "oth_dump13.txt", vec!(("example.wav".to_string(), "E57CC94793F1A408226B070046C2D6253E108C4A".to_string()),
                                           ("temper_in_°.txt".to_string(), "49584B5C027111E7A9F8F04BED3550A7FAA41DA4".to_string())),
                    vec!(("file_1".to_string(), "example.wav".to_string()))
                       );
    }
    use std::path::Path;
    fn create_file(name: &str, content: &str) {
        let path = Path::new(name);
        let directory = path.parent().expect("Error when calling parent()");
        fs::create_dir_all(directory).expect("Error when creating directories");
        let mut output = File::create(name).expect("Unable to create file");
        output.write(format!("{}", content).as_bytes()).expect("Error during file write");
    }
    fn create_filetree(root: &str, list: Vec::<(String, String)>) {
        list.iter().for_each(|(name, content)| {
            let mut filename = PathBuf::new();
            filename.push(root);
            filename.push(name);
            create_file(&filename.to_str().expect("error during PathBuf::to_str()"), content);
        });
    }
    #[test]
    fn test_compare_trees() {
        let ref_name = "ref3";
        let oth_name = "oth3";
        create_filetree(ref_name, vec!(("dummy_dir1/dummy_dur2/test.txt".to_string(), "This is a dummy file".to_string()),
                                     ("dummy_dir1/dummy_dur2/test2.txt".to_string(), "This is a an other dummy file".to_string()),
                                    ));
        create_filetree(oth_name, vec!(("similar/a.txt".to_string(), "This is a dummy file".to_string()),
                                     ("similar/b.txt".to_string(), "This is a an other dummy file".to_string()),
                                     ("c.txt".to_string(), "This is a an other dummy file".to_string()),
                                    ));
        assert_eq!(vec!(("ref3/dummy_dir1/dummy_dur2".to_string(), "oth3/similar".to_string()),
                        ("ref3/dummy_dir1/dummy_dur2/test2.txt".to_string(), "oth3/c.txt".to_string())
                       ), compare_trees(ref_name, oth_name).expect("Error during comparison"));
        assert!(fs::remove_dir_all(ref_name).is_ok());
        assert!(fs::remove_dir_all(oth_name).is_ok());
        assert!(fs::remove_dir_all(dump_dir(ref_name)).is_ok());
        assert!(fs::remove_dir_all(dump_dir(oth_name)).is_ok());
        assert!(fs::remove_file(dump_name(ref_name)).is_ok());
        assert!(fs::remove_file(dump_name(oth_name)).is_ok());
    }
    #[test]
    fn test_compare_trees2() {
        let ref_name = "ref4";
        let oth_name = "oth4";
        create_filetree(ref_name, vec!(("dummy_dir1/dummy_dur2/test.txt".to_string(), "This is a dummy file".to_string()),
                                       ("dummy_dir1/dummy_dur2/test2.txt".to_string(), "This is a an other dummy file".to_string()),
                                       ("dummy_dir1/c.txt".to_string(), "This is a an other dummy file".to_string()),
                                      ));
        create_filetree(oth_name, vec!(("similar/a.txt".to_string(), "This is a dummy file".to_string()),
                                       ("similar/b.txt".to_string(), "This is a an other dummy file".to_string()),
                                       ("c.txt".to_string(), "This is a an other dummy file".to_string()),
                                      ));
        assert_eq!(vec!(("ref4/dummy_dir1".to_string(), "oth4".to_string())), compare_trees(ref_name, oth_name).expect("Error during comparison"));
        assert!(fs::remove_dir_all(ref_name).is_ok());
        assert!(fs::remove_dir_all(oth_name).is_ok());
        assert!(fs::remove_dir_all(dump_dir(ref_name)).is_ok());
        assert!(fs::remove_dir_all(dump_dir(oth_name)).is_ok());
        assert!(fs::remove_file(dump_name(ref_name)).is_ok());
        assert!(fs::remove_file(dump_name(oth_name)).is_ok());
    }
    #[test]
    fn test_compare_trees3() {
        let ref_name = "ref5";
        let oth_name = "oth5";
        create_filetree(ref_name, vec!(("dummy_dir1/dummy_dur2/test.txt".to_string(), "This is a dummy file".to_string()),
                                       ("dummy_dir1/dummy_dur2/test2.txt".to_string(), "This is a an other dummy file".to_string()),
                                       ("dummy_dir1/c.txt".to_string(), "This is a an other dummy file".to_string()),
                                      ));
        create_filetree(oth_name, vec!(("dir/similar/a.txt".to_string(), "This is a dummy file".to_string()),
                                       ("dir/similar/b.txt".to_string(), "This is a an other dummy file".to_string()),
                                       ("dir/c.txt".to_string(), "This is a an other dummy file".to_string()),
                                       ("similar_bis/a.txt".to_string(), "This is a dummy file".to_string()),
                                       ("similar_bis/b.txt".to_string(), "This is a an other dummy file".to_string()),
                                      ));
        assert_eq!(vec!(("ref5/dummy_dir1".to_string(), "oth5/dir".to_string()), ("ref5/dummy_dir1/dummy_dur2".to_string(), "oth5/similar_bis".to_string())), compare_trees(ref_name, oth_name).expect("Error during comparison"));
        assert!(fs::remove_dir_all(ref_name).is_ok());
        assert!(fs::remove_dir_all(oth_name).is_ok());
        assert!(fs::remove_dir_all(dump_dir(ref_name)).is_ok());
        assert!(fs::remove_dir_all(dump_dir(oth_name)).is_ok());
        assert!(fs::remove_file(dump_name(ref_name)).is_ok());
        assert!(fs::remove_file(dump_name(oth_name)).is_ok());
    }
    #[test]
    fn test_compare_trees4() {
        let ref_name = "ref6";
        let oth_name = "oth6";
        create_filetree(ref_name, vec!(("dummy_dir1/dummy_dur2/test.txt".to_string(), "This is a dummy file".to_string()),
                                       ("dummy_dir1/dummy_dur2/test2.txt".to_string(), "This is a an other dummy file".to_string()),
                                       ("dummy_dir1/break.txt".to_string(), "This is a file to break arborescence matching".to_string()),
                                       ("dummy_dir1/c.txt".to_string(), "This is yet an other dummy file".to_string()),
                                      ));
        create_filetree(oth_name, vec!(("dir/similar/a.txt".to_string(), "This is a dummy file".to_string()),
                                       ("dir/similar/b.txt".to_string(), "This is a an other dummy file".to_string()),
                                       ("dir/similar/disturb.txt".to_string(), "This is a file to break arborescence matching".to_string()),
                                       ("dir/c.txt".to_string(), "This is yet an other dummy file".to_string()),
                                       ("similar_bis/a.txt".to_string(), "This is a dummy file".to_string()),
                                       ("similar_bis/b.txt".to_string(), "This is a an other dummy file".to_string()),
                                      ));
        assert_eq!(vec!(("ref6/dummy_dir1/dummy_dur2".to_string(), "oth6/similar_bis".to_string()),
                        ("ref6/dummy_dir1/dummy_dur2/test2.txt".to_string(), "oth6/dir/similar/b.txt".to_string()),
                        ("ref6/dummy_dir1/c.txt".to_string(), "oth6/dir/c.txt".to_string()),
                        ("ref6/dummy_dir1/break.txt".to_string(), "oth6/dir/similar/disturb.txt".to_string()),
                        ("ref6/dummy_dir1/dummy_dur2/test.txt".to_string(), "oth6/dir/similar/a.txt".to_string())
                        ), compare_trees(ref_name, oth_name).expect("Error during comparison"));
        assert!(fs::remove_dir_all(ref_name).is_ok());
        assert!(fs::remove_dir_all(oth_name).is_ok());
        assert!(fs::remove_dir_all(dump_dir(ref_name)).is_ok());
        assert!(fs::remove_dir_all(dump_dir(oth_name)).is_ok());
        assert!(fs::remove_file(dump_name(ref_name)).is_ok());
        assert!(fs::remove_file(dump_name(oth_name)).is_ok());
    }
}

