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

#[derive(Debug)]
pub struct FileTreeInfo {
    pub name: String,
    pub height: u32
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn create_filetree_info() {
        let ref_filetree_info = FileTreeInfo {
            name: "filetree".to_string(),
            height: 8
        };
        assert_eq!(ref_filetree_info.name, "filetree");
        assert_eq!(ref_filetree_info.height, 8);
    }

}
