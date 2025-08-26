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

#[derive(Debug)]
#[derive(PartialEq)]
pub struct FileTreeInfo {
    pub name: String,
    pub height: u32,
    pub sha1: sha1::Sha1Key
}

impl fmt::Display for FileTreeInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}, {}", self.sha1, self.name, self .height)
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
            sha1: sha1::compute_sha1(vec!(0))
        };
        assert_eq!(ref_filetree_info.name, "filetree");
        assert_eq!(ref_filetree_info.height, 8);
        assert_eq!(ref_filetree_info.sha1, sha1::compute_sha1(vec!(0)));
    }

    #[test]
    fn check_filetree_info_display() {
        let ref_filetree_info = FileTreeInfo {
            name: "filetree".to_string(),
            height: 8,
            sha1: sha1::compute_sha1(vec!(0))
        };
        assert_eq!(format!("{}", ref_filetree_info), "EDA2784F420E43F652B521D7B0CFF93F5BA93C9D filetree, 8");
    }
}
