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
        println!("Parse existing dump for {}", name);
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
                println!("==> Line '{}'", line);
            }
            else {
                return Err(format!("Unable to read from {}", dump_name(name)));
            }
        }
        let filetree_info = filetree_info::FileTreeInfo::from(&line)?;
        result = filetree_info.height;
    }
    else {
        println!("Generate dump for {}", name);
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
        println!("==> Prepare split");
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
                    println!("==> Create split {filename}");
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
                    println!("==> Dispatching '{}'", line);
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
                println!("==> Sort split {filename}");
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
                to_remove: &mut Vec<String>) -> Result<(), String> {
    // File can never be empty as splitted files are created when encoutering a FileTreeInfo
    let mut ref_item = filetree_info::FileTreeInfo::from(&consume(&mut reference)?)?;
    let mut other_line = consume(&mut other)?;
    loop {
        // Check if we reach end of one of the files
        if other_line == "" {
            return Ok(())
        }
        let other_item = filetree_info::FileTreeInfo::from(&other_line)?;
        if to_remove.iter().any(|x| x.len() <= other_item.name.len() && x == &other_item.name[0..x.len()]) {
            other_line = consume(&mut other)?;
            continue;
        }
        if ref_item.equivalent(&other_item) {
            to_remove.push(other_item.name);
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

fn compare(reference: &str, other: &str, height: u32) -> Result<Vec<String>, String> {
    let mut to_remove = Vec::new();
    for i in (0..height + 1).rev() {
        println!("=>Analyse height {}", i);
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

pub fn run(configuration: &Config) -> Result<(), Box<dyn Error>> {
    println!("Reference path {}", configuration.reference_path);
    println!("Other path {}", configuration.other_path);

    let result = check_directory(&configuration.reference_path);
    if result.is_err() {
        return Err(result.err().unwrap().into());
    }
    let result = check_directory(&configuration.other_path);
    if result.is_err() {
        return Err(result.err().unwrap().into());
    }

    let height_ref = generate_dump(&configuration.reference_path)?;
    let height_other = generate_dump(&configuration.other_path)?;
    println!("Dump result {} vs {}", height_ref, height_other);

    let common_height = if height_ref > height_other {height_other} else {height_ref};
    println!("Comparison will be done until height {}", common_height);

    generate_split(&configuration.reference_path, height_ref)?;
    generate_split(&configuration.other_path, height_other)?;

    let result = compare(&configuration.reference_path, &configuration.other_path, common_height)?;

    result.iter().for_each(|s| println!("TO REMOVE {}", s));

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
            height: 0,
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
        assert_eq!(2, analyse("reference2").expect("Error with reference").height);
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
                        vec!("tutu".to_string())
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
        let mut to_remove = Vec::<String>::new();
        compare_iter(ref_bufreader.lines(), other_bufreader.lines(), &mut to_remove).expect("Error during comparison");
        assert_eq!(Vec::<String>::new(), to_remove);
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
                       ref_to_remove: Vec<String>
                      ) {
        create_dump(ref_name, ref_list);
        create_dump(oth_name, oth_list);
        let ref_dump = File::open(ref_name).expect("Unable to open ref dump");
        let oth_dump = File::open(oth_name).expect("Unable to open other dump");
        let ref_bufreader = BufReader::new(ref_dump);
        let oth_bufreader = BufReader::new(oth_dump);
        let mut to_remove = Vec::<String>::new();
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
                        vec!("other1".to_string())
                       );
    }
    #[test]
    fn test_compare_ref2() {
        compare_generic("ref_dump6.txt", vec!(("file_1".to_string(), "E57CC94793F1A408226B070046C2D6253E108C4A".to_string()),
                                              ("file_2".to_string(), "2CC944E46E5029A3AAFFE9554CD950C3C79694CC".to_string()),
                                              ("file_3".to_string(), "49584B5C027111E7A9F8F04BED3550A7FAA41DA4".to_string())
                                             ),
                        "oth_dump6.txt", vec!(("other2".to_string(), "2CC944E46E5029A3AAFFE9554CD950C3C79694CC".to_string())),
                        vec!("other2".to_string())
                       );
    }
    #[test]
    fn test_compare_ref3() {
        compare_generic("ref_dump7.txt", vec!(("file_1".to_string(), "E57CC94793F1A408226B070046C2D6253E108C4A".to_string()),
                                              ("file_2".to_string(), "2CC944E46E5029A3AAFFE9554CD950C3C79694CC".to_string()),
                                              ("file_3".to_string(), "49584B5C027111E7A9F8F04BED3550A7FAA41DA4".to_string())
                                             ),
                        "oth_dump7.txt", vec!(("other3".to_string(), "49584B5C027111E7A9F8F04BED3550A7FAA41DA4".to_string())),
                        vec!("other3".to_string())
                       );
    }
    #[test]
    fn test_compare_other1() {
        compare_generic("ref_dump8.txt", vec!(("file_1".to_string(), "E57CC94793F1A408226B070046C2D6253E108C4A".to_string())),
                        "oth_dump8.txt", vec!(("other1".to_string(), "E57CC94793F1A408226B070046C2D6253E108C4A".to_string()),
                                              ("other2".to_string(), "2CC944E46E5029A3AAFFE9554CD950C3C79694CC".to_string()),
                                              ("other3".to_string(), "49584B5C027111E7A9F8F04BED3550A7FAA41DA4".to_string())),
                        vec!("other1".to_string())
                       );
    }
    #[test]
    fn test_compare_other2() {
        compare_generic("ref_dump9.txt", vec!(("file_1".to_string(), "2CC944E46E5029A3AAFFE9554CD950C3C79694CC".to_string())),
                        "oth_dump9.txt", vec!(("other1".to_string(), "E57CC94793F1A408226B070046C2D6253E108C4A".to_string()),
                                              ("other2".to_string(), "2CC944E46E5029A3AAFFE9554CD950C3C79694CC".to_string()),
                                              ("other3".to_string(), "49584B5C027111E7A9F8F04BED3550A7FAA41DA4".to_string())),
                        vec!("other2".to_string())
                       );
    }
    #[test]
    fn test_compare_other3() {
        compare_generic("ref_dump10.txt", vec!(("file_1".to_string(), "49584B5C027111E7A9F8F04BED3550A7FAA41DA4".to_string())),
                        "oth_dump10.txt", vec!(("other1".to_string(), "E57CC94793F1A408226B070046C2D6253E108C4A".to_string()),
                                               ("other2".to_string(), "2CC944E46E5029A3AAFFE9554CD950C3C79694CC".to_string()),
                                               ("other3".to_string(), "49584B5C027111E7A9F8F04BED3550A7FAA41DA4".to_string())),
                        vec!("other3".to_string())
                       );
    }
    #[test]
    fn test_compare_several_other1() {
        compare_generic("ref_dump11.txt", vec!(("file_1".to_string(), "49584B5C027111E7A9F8F04BED3550A7FAA41DA4".to_string())),
                        "oth_dump11.txt", vec!(("other1".to_string(), "E57CC94793F1A408226B070046C2D6253E108C4A".to_string()),
                                               ("other2".to_string(), "49584B5C027111E7A9F8F04BED3550A7FAA41DA4".to_string()),
                                               ("other3".to_string(), "49584B5C027111E7A9F8F04BED3550A7FAA41DA4".to_string())),
                        vec!("other2".to_string(), "other3".to_string())
                       );
    }
    #[test]
    fn test_compare_several_other2() {
        compare_generic("ref_dump12.txt", vec!(("file_1".to_string(), "E57CC94793F1A408226B070046C2D6253E108C4A".to_string())),
                        "oth_dump12.txt", vec!(("other1".to_string(), "E57CC94793F1A408226B070046C2D6253E108C4A".to_string()),
                                               ("other2".to_string(), "E57CC94793F1A408226B070046C2D6253E108C4A".to_string()),
                                               ("other3".to_string(), "49584B5C027111E7A9F8F04BED3550A7FAA41DA4".to_string())),
                        vec!("other1".to_string(), "other2".to_string())
                       );
    }
}

