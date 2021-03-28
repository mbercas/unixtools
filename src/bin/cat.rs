/*
 * Title: Unix `cat` command implemented in Rust
 * Author: Manuel Berrocal
 * email: mbercas@gmail.com
 *
 * Description:
 * The cat commands opens a file and writes its contents into the standard output.
 *
 *
 * Exit error codes:
 *   1: invalid path to input file
 *   2: can not open input file for reading
 */

/*
 * This work is licensed under a Creative Commons Attribution 3.0 Unported License
 * http://creativecommons.org/licenses/by/3.0/deed.en_US"
 */

/*
 * References:
 * - Parsing command line:
 *     - https://docs.rs/clap/2.33.3/clap/
 *     - https://rust-lang-nursery.github.io/rust-cookbook/cli/arguments.html
 * - File::IO
 *     - https://doc.rust-lang.org/rust-by-example/std_misc/file/open.html
 *     - https://doc.rust-lang.org/rust-by-example/std_misc/file/read_lines.html
 * - Process Exit
 *      - https://stackoverflow.com/questions/21569718/how-do-i-exit-a-rust-program-early-from-outside-the-main-function
 *
 */
use clap::{App, Arg};

use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::process;

const VERSION: &str = "ver. 0.0.1";

/**
 * Exit codes, note that Process::exit requires i32 as argument
 */
enum Rc {
    ErrorInvalidIinputFilePath = 1,
    ErrorCannotOpenFileForReading = 2,
}

/**
 * A structure that defines how the output is formatted.
 */
struct OutputFormatter {
    has_line_numbers: bool,
    only_non_blank: bool,
    squeze_blank: bool,
    next_line_number: u32,
}

impl OutputFormatter {
    fn increase_line_number(&mut self) {
        self.next_line_number += 1;
    }
}

fn main() {
    // Initialize the output formatter
    let mut output_formatter = OutputFormatter {
        has_line_numbers: false,
        only_non_blank: false,
        squeze_blank: false,
        next_line_number: 0,
    };

    let matches = App::new("rcat: cat clone command written in Rust")
        .version(VERSION)
        .author("Manuel Berrocal")
        .about("Write concatenated file contents into standard output")
        .arg(
            Arg::with_name("number")
                .short("n")
                .long("number")
                .takes_value(false)
                .help("number all output lines"),
        )
        .arg(
            Arg::with_name("number-nonblank")
                .short("b")
                .long("number-nonblank")
                .takes_value(false)
                .help("number only non blank lines"),
        )
        .arg(
            Arg::with_name("squeze-blank")
                .short("s")
                .long("squeze-blank")
                .takes_value(false)
                .help("suppress repeated blank lines"),
        )
        .arg(
            Arg::with_name("inputs")
                .help("Input files")
                .required(true)
                .takes_value(true)
                .multiple(true),
        )
        .get_matches();

    // This is only safe because the argument is required.
    let inputs: Vec<_> = matches.values_of("inputs").unwrap().collect();
    println!("Value of input: {:?}", inputs);

    if matches.is_present("number") {
        output_formatter.has_line_numbers = true;
    }

    if matches.is_present("number-nonblank") {
        output_formatter.has_line_numbers = true;
        output_formatter.only_non_blank = true;
    }

    if matches.is_present("squeze-blank") {
        output_formatter.squeze_blank = true;
    }

    // Verify that files in list of input files exist
    let mut file_paths = Vec::new();
    for file_name in &inputs {
        let path = Path::new(file_name);
        if !path.exists() {
            eprintln!("ERROR: file: `{}` does not exist", path.display());
            process::exit(Rc::ErrorInvalidIinputFilePath as i32);
        }
        file_paths.push(path);
    }

    // For every file verify that the  file descriptor can be opened
    // we don't want to generate partial output if one of the files
    // can not be opened.
    //
    // We could store the file descriptors in an array but does not
    // make sense to have too many file descriptors open
    for file_path in &file_paths {
        let _ = match File::open(&file_path) {
            Err(err_code) => {
                eprintln!(
                    "ERROR opening file `{}` for reading: {}",
                    file_path.display(),
                    err_code
                );
                process::exit(Rc::ErrorCannotOpenFileForReading as i32);
            }
            Ok(file) => file,
        };
    }

    // For every file read the contents
    for file_path in &file_paths {
        let lines = match File::open(&file_path) {
            Err(err_code) => {
                eprintln!(
                    "ERROR opening file `{}` for reading: {}",
                    file_path.display(),
                    err_code
                );
                process::exit(Rc::ErrorCannotOpenFileForReading as i32);
            }
            Ok(file) => io::BufReader::new(file).lines(),
        };
        let mut prev_blank = false;
        for line in lines {
            if let Ok(ok_line) = line {
                let is_blank = ok_line.trim() == "";

                if !is_blank | (is_blank & !output_formatter.only_non_blank) {
                    output_formatter.increase_line_number();
                }

                if output_formatter.squeze_blank & (prev_blank & is_blank) {
                    continue;
                }
                prev_blank = is_blank;

                println!(
                    "{}{}",
                    if is_blank & output_formatter.only_non_blank {
                        format!("{:<5}: ", String::from(""))
                    } else if output_formatter.has_line_numbers {
                        format!("{:<5}: ", output_formatter.next_line_number)
                    } else {
                        String::from("")
                    },
                    ok_line
                )
            }
        }
    }
}
