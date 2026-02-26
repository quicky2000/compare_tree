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
use std::fs::File;
use std::io::BufWriter;
use std::io::Write;

use crate::output_module::OutputModule;
use crate::ct_utils::despecialise;

pub struct BatchModule {
      filename: String,
      output_file: BufWriter<File>
}

fn dump_duplicated(output_file: &mut BufWriter<File>, reference: &str, other: &str) -> Result<(), std::io::Error> {
            let keep = despecialise(reference);
            let remove = despecialise(other);
            output_file.write(format!("if [ ! -L {} -a -f {} ]\n", keep, keep).as_bytes())?;
            output_file.write("then\n".as_bytes())?;
            output_file.write(format!("    rm {}\n", remove).as_bytes())?;
            output_file.write(format!("elif [ -L {}  ]\n", keep).as_bytes())?;
            output_file.write("then\n".as_bytes())?;
            output_file.write(format!(r#"    echo "{}" is a link{}"#, keep, "\n").as_bytes())?;
            output_file.write("else\n".as_bytes())?;
            output_file.write(format!(r#"    echo "{}" do not exist{}"#, keep, "\n").as_bytes())?;
            output_file.write("fi\n\n".as_bytes())?;
            Ok(())
}

impl OutputModule for BatchModule {
      fn treat_internal_doublon(&mut self, first: &str, second: &str) {
            self.output_file.write(format!("# Doublon {} <-> {}\n\n", despecialise(first), despecialise(second)).as_bytes()).expect(format!("Unable to write in file {}", self.filename).as_str());
      }
      fn treat_duplicated(&mut self, reference: &str, other: &str) -> Result<bool, String> {
            dump_duplicated(&mut self.output_file, reference, other).expect(format!("Error during write of file {}", self.filename).as_str());
            Ok(true)
      }
}

impl Drop for BatchModule {
    fn drop(&mut self) {
        self.output_file.write("#EOF\n".as_bytes()).expect(format!("Unable to write in file {}", self.filename).as_str());
    }
}

impl BatchModule {
      pub fn new() -> BatchModule {
            let filename = "batch.zsh";
            let file = File::create(&filename).expect(format!("Unable to create file {}", filename).as_str());
            let mut buf = BufWriter::new(file);
            buf.write("#!/bin/bash\n\n".as_bytes()).expect(format!("Unable to write in file {}", filename).as_str());
            BatchModule { filename: filename.to_string(), output_file: buf}
      }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fs;
    use std::io::Read;

        #[test]
    fn check_batch_module() {
          let ref_name = "ref.zsh";
          let reference = r#"#!/bin/bash

# Doublon first_file <-> second_file

if [ ! -L original -a -f original ]
then
    rm duplicated
elif [ -L original  ]
then
    echo "original" is a link
else
    echo "original" do not exist
fi

#EOF
"#;
          {
                let mut my_module = BatchModule::new();
                my_module.treat_internal_doublon("first_file", "second_file");
                let _ = my_module.treat_duplicated("original", "duplicated");

                // Dump ref file to make diff easier in case of mismatch
                let file = File::create(&ref_name).expect(format!("Unable to create file {}", ref_name).as_str());
                let mut buf = BufWriter::new(file);
                buf.write(reference.as_bytes()).expect(format!("Unable to write in file {}", ref_name).as_str());
          }
          let batch_name = "batch.zsh";
          let mut batch_file = File::open(batch_name).expect(format!("Unable to open file {} for read", batch_name).as_str());
          let mut contents = String::new();
          batch_file.read_to_string(&mut contents).expect(format!("Unable to read content of file {}", batch_name).as_str());
          assert_eq!(contents, reference);
          assert!(fs::remove_file(batch_name).is_ok());
          assert!(fs::remove_file(ref_name).is_ok());
    }
}
