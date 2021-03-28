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

#[derive(Clone, Copy)]
struct OutputFormatter {
    has_line_numbers: bool,
    only_non_blank: bool,
    squeze_blank: bool,
    ignore_errors: bool,
}

/**
 * Read the command line arguments and parse them into the OutputFormatter
 * structure. Return input files in a vector.
 */
fn read_arguments() -> (OutputFormatter, Vec<String>) {
    let mut output_formatter = OutputFormatter {
        has_line_numbers: false,
        only_non_blank: false,
        squeze_blank: false,
        ignore_errors: false,
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
            Arg::with_name("ignore-errors")
                .short("i")
                .long("ignore-errors")
                .takes_value(false)
                .help("Ignore errors that affect invidiual files"),
        )
        .arg(
            Arg::with_name("inputs")
                .help("Input files")
                .required(true)
                .takes_value(true)
                .multiple(true),
        )
        .get_matches();

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

    if matches.is_present("ignore-errors") {
        output_formatter.ignore_errors = true;
    }

    //(output_formatter, inputs)
    // This is only safe because the argument is required.
    let mut inputs = Vec::new();
    let tmp: Vec<_> = matches.values_of("inputs").unwrap().collect();
    for file_name in tmp {
        inputs.push(file_name.to_string())
    }

    (output_formatter, inputs)
}

/**
 * Check that the list of strings passed as an argument describes valid paths.
 */
fn get_file_paths(inputs: &Vec<String>, ignore_errors: bool) -> Result<Vec<&Path>, Rc> {
    let mut file_paths = Vec::new();
    for file_name in inputs {
        let path = Path::new(file_name.as_str());
        if !path.exists() {
            eprintln!("ERROR: file: `{}` does not exist", path.display());
            if !ignore_errors {
                process::exit(Rc::ErrorInvalidIinputFilePath as i32);
            }
        }
        file_paths.push(path);
    }
    Ok(file_paths)
}

fn main() {
    let (output_formatter, inputs) = read_arguments();

    let file_paths = match get_file_paths(&inputs, output_formatter.ignore_errors) {
        Ok(file_paths) => file_paths,
        Err(rc) => {
            process::exit(rc as i32);
        }
    };

    // For every file read the contents
    let mut next_line_number = 0u32;
    for file_path in &file_paths {
        let lines = match File::open(&file_path) {
            Err(err_code) => {
                eprintln!(
                    "ERROR opening file `{}` for reading: {}",
                    file_path.display(),
                    err_code
                );
                if output_formatter.ignore_errors {
                    continue;
                } else {
                    process::exit(Rc::ErrorCannotOpenFileForReading as i32);
                }
            }
            Ok(file) => io::BufReader::new(file).lines(),
        };
        let mut prev_blank = false;
        for line in lines {
            if let Ok(ok_line) = line {
                let is_blank = ok_line.trim() == "";

                if !is_blank | (is_blank & !output_formatter.only_non_blank) {
                    next_line_number += 1;
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
                        format!("{:<5}: ", next_line_number)
                    } else {
                        String::from("")
                    },
                    ok_line
                )
            }
        }
    }
}
