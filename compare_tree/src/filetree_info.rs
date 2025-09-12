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

use std::fmt;
use crate::sha1;
use std::str::FromStr;

#[derive(Debug)]
#[derive(PartialEq, PartialOrd, Eq, Ord)]
pub struct FileTreeInfo {
    pub sha1: sha1::Sha1Key,
    pub height: u32,
    pub nb_item: u32,
    pub name: String
}

impl fmt::Display for FileTreeInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}, {}, {}", self.sha1, self.name, self.height, self.nb_item)
    }
}

impl FileTreeInfo {
    pub fn from(v: &str) -> Result<FileTreeInfo, String> {
        let space_pos_result = v.find(' ');
        let space_pos = match space_pos_result {
            Some(i) => i,
            None => return Err(format!("Space not found in {}", v))
        };

        let last_comma_pos_result = v.rfind(", ");
        let last_comma_pos = match last_comma_pos_result {
            Some(i) => i,
            None => return Err(format!("Last ',' not found in {}", v))
        };
        let nb_item_slice = &v[last_comma_pos + 2..];
        let nb_item_result = u32::from_str(nb_item_slice);
        let nb_item = match nb_item_result {
            Ok(v) => v,
            Err(e) => return Err(format!("Filetree_info.nb_item : Error {} when converting {} to u32", e, nb_item_slice))
        };
        let height_comma_pos_result = v[0..last_comma_pos - 1].rfind(", ");
        let height_comma_pos = match height_comma_pos_result {
            Some(i) => i,
            None => return Err(format!("',' following name not found in {}", v))
        };
        let height_slice = &v[height_comma_pos + 2..last_comma_pos];
        let height_result = u32::from_str(height_slice);
        let height = match height_result {
            Ok(v) => v,
            Err(e) => return Err(format!("Filetree_info.height : Error {} when converting {} to u32 {} {}", e, height_slice, height_comma_pos, last_comma_pos))
        };
        let result = FileTreeInfo {
            name: v[space_pos + 1..height_comma_pos].to_string(),
            height: height,
            sha1: sha1::Sha1Key::from_string(&v[0..space_pos])?,
            nb_item: nb_item
        };
        Ok(result)

    }
    pub fn equivalent(&self, op2: &FileTreeInfo) -> bool {
        self.height == op2.height && self.sha1 == op2.sha1 && self.nb_item == op2.nb_item
    }

}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn create_filetree_info() {
        let ref_filetree_info = FileTreeInfo {
            name: "filetree".to_string(),
            height: 8,
            sha1: sha1::compute_sha1(vec!(0)),
            nb_item: 10
        };
        assert_eq!(ref_filetree_info.name, "filetree");
        assert_eq!(ref_filetree_info.height, 8);
        assert_eq!(ref_filetree_info.sha1, sha1::compute_sha1(vec!(0)));
        assert_eq!(ref_filetree_info.nb_item, 10);
    }

    #[test]
    fn check_filetree_info_display() {
        let ref_filetree_info = FileTreeInfo {
            name: "filetree".to_string(),
            height: 8,
            sha1: sha1::compute_sha1(vec!(0)),
            nb_item: 10
        };
        assert_eq!(format!("{}", ref_filetree_info), "EDA2784F420E43F652B521D7B0CFF93F5BA93C9D filetree, 8, 10");
    }
    #[test]
    fn check_filetree_info_order() {
        let filetree_info1 = FileTreeInfo {
            name: "a".to_string(),
            height: 1,
            nb_item: 0,
            sha1: sha1::compute_sha1(vec!(1))
        };
        let filetree_info2 = FileTreeInfo {
            name: "b".to_string(),
            height: 1,
            nb_item: 0,
            sha1: sha1::compute_sha1(vec!(0))
        };
        print!("{:?}\n{:?}", filetree_info1, filetree_info2);
        assert!(filetree_info1 > filetree_info2);
        let filetree_info3 = FileTreeInfo {
            name: "z".to_string(),
            height: 1,
            nb_item: 0,
            sha1: sha1::compute_sha1(vec!(0))
        };
        let filetree_info4 = FileTreeInfo {
            name: "b".to_string(),
            height: 2,
            nb_item: 0,
            sha1: sha1::compute_sha1(vec!(0))
        };
        assert!(filetree_info3 < filetree_info4);
        let filetree_info5 = FileTreeInfo {
            name: "a".to_string(),
            height: 2,
            nb_item: 7,
            sha1: sha1::compute_sha1(vec!(0))
        };
        let filetree_info6 = FileTreeInfo {
            name: "a".to_string(),
            height: 2,
            nb_item: 5,
            sha1: sha1::compute_sha1(vec!(0))
        };
        assert!(filetree_info5 > filetree_info6);
        let filetree_info7 = FileTreeInfo {
            name: "b".to_string(),
            height: 2,
            nb_item: 7,
            sha1: sha1::compute_sha1(vec!(0))
        };
        let filetree_info8 = FileTreeInfo {
            name: "a".to_string(),
            height: 2,
            nb_item: 7,
            sha1: sha1::compute_sha1(vec!(0))
        };
        assert!(filetree_info7 > filetree_info8);
    }
    #[test]
    fn check_from_string() {
        let ref_filetree_info = FileTreeInfo {
            name: "filetree".to_string(),
            height: 8,
            sha1: sha1::compute_sha1(vec!(0)),
            nb_item: 10
        };
        assert_eq!( ref_filetree_info, FileTreeInfo::from("EDA2784F420E43F652B521D7B0CFF93F5BA93C9D filetree, 8, 10").expect("Error during string conversion"));
    }
}
