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

use crate::output_module::OutputModule;
use std::fs;
use std::io;
use crate::ct_utils::despecialise;

pub struct InteractiveModule {
}

impl OutputModule for InteractiveModule {
      fn treat_internal_doublon(&mut self, first: &str, second: &str) {
            eprintln!("!!! Doublon {} <-> {}", first, second);
            println!("What to do ? (rf/rs/s)");
            let mut answer = String::new();
            while answer == "" {
                  io::stdin().read_line(&mut answer).expect("Failed to read line");
                  let len = answer.chars().count();
                  if len > 1 && answer.chars().nth(len - 1) == Some('\n') {
                        answer = answer.chars().take(len - 1).collect::<String>();
                  }
                  println!("Your answer is '{}'", answer);
                  if answer == "rf".to_string() {
                        fs::remove_file(&first).expect("Unable to remove file");
                        break;
                  }
                  else if answer == "rs".to_string() {
                        fs::remove_file(&second).expect("Unable to remove file");
                        break;
                  }
                  else if answer == "s".to_string() {
                        break;
                  }
            }
      }

      fn treat_duplicated(&mut self, reference: &str, other: &str) -> Result<bool, String> {
            eprintln!("{} TO REMOVE {}", reference, other);
            let exist_ref_result = fs::exists(&reference);
            let exist_ref = match exist_ref_result {
                Ok(r) => r,
                Err(e) => return Err(format!("Error when trying to check if {} exists : {}", reference, e).into())
            };
            let exist_oth_result = fs::exists(&other);
            let exist_oth = match exist_oth_result {
                  Ok(r) => r,
                  Err(e) => return Err(format!("Error when trying to check if {} exists : {}", other, e).into())
            };

            if exist_ref && exist_oth {
                  eprintln!("{} TO REMOVE {}", despecialise(&reference), despecialise(&other));
                  println!("rm {} ? (y/n/q)", despecialise(&other));
                  let mut answer = String::new();
                  while answer == "" {
                        io::stdin().read_line(&mut answer).expect("Failed to read line");
                        let len = answer.chars().count();
                        if len > 1 && answer.chars().nth(len - 1) == Some('\n') {
                              answer = answer.chars().take(len - 1).collect::<String>();
                        }
                        println!("Your answer is '{}'", answer);
                        if answer == "y".to_string() {
                              fs::remove_file(&other).expect("Unable to remove file");
                              break;
                        }
                        else if answer == "n".to_string() {
                              break;
                        }
                        else if answer == "q".to_string() {
                              return Ok(false);
                        }
                  }
            }
            Ok(true)
      }
}
