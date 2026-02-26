# Compare_tree

Continuous integration with [Travis-Ci](https://travis-ci.org/quicky2000/compare_tree) : ![Build Status](https://travis-ci.org/quicky2000/compare_tree.svg?branch=master)

Please see LICENSE for info on the license.

This tool detect tree duplications between 2 trees and will indicate if some trees appear several times

## How to use it

`./compare_tree <reference directory> <other director> [-p | -i | -b]`

* -p : print mode, display information about duplicated
* -i : interactive mode, user is asked which file to remove
* -b : generate a script that will remove from other directory duplicated files that are in reference directory

