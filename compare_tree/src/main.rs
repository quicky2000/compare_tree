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
use std::env::args;
use std::process;
use compare_tree::Config;

fn main() {
    println!("Hello, world!");
    let args = args();
    dbg!(&args);


    let configuration = Config::build(args).unwrap_or_else(|err| {
        eprintln!("Error when parsing aguments : {err}");
        process::exit(-1);
    });

    if let Err(error) = compare_tree::run(&configuration) {
        eprintln!("Error occurr {error}");
        process::exit(-1);
    }
}

