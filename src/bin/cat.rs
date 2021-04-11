///
/// Title: Unix `cat` command implemented in Rust
/// Author: Manuel Berrocal
/// email: mbercas@gmail.com
///
///  # Description:
///  The cat commands opens a file and writes its contents into the standard output.
///
///
///  Exit error codes:
///  * 1: invalid path to input file
///  * 2: can not open input file for reading
/*
* References:
* - Writing CLI applications
      - https://rust-cli.github.io/book/index.html
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
use std::io::{self, BufRead, Write};
use std::process;

extern crate toolslib;

const VERSION: &str = "ver. 0.0.2";

/// A structure that defines how the output is formatted.
struct OutputFormatter {
    has_line_numbers: bool,
    only_non_blank: bool,
    squeze_blank: bool,
    ignore_errors: bool,
    inputs: Vec<String>,
}

impl OutputFormatter {
    /// Initialize the OutputFormatter with defaults
    fn new() -> OutputFormatter {
        OutputFormatter {
            has_line_numbers: false,
            only_non_blank: false,
            squeze_blank: false,
            ignore_errors: false,
            inputs: Vec::new(),
        }
    }
}

/// Read the command line arguments and parse them into the OutputFormatter
/// structure.
fn read_arguments() -> OutputFormatter {
    let mut output_formatter = OutputFormatter::new();
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

    // This is only safe because the argument is required.

    let tmp: Vec<_> = matches.values_of("inputs").unwrap().collect();
    for file_name in tmp {
        output_formatter.inputs.push(file_name.to_string());
    }

    output_formatter
}

/// Returns a string with the formated line
///
/// # Arguments
///
/// * `line` - String to be formatted
/// * `line_number` - u32 the line number to append to the line
/// * `output_formatter` - OutputFormatter structure containing the formatting parameters
///
/// Appends a number to the line if the -n switch was passed in the command line arguments.
/// Ignores blank lines if -b switch was passsed in the command line arguments.
///
fn format_output_line(
    line: &String,
    line_number: u32,
    output_formatter: &OutputFormatter,
) -> String {
    let is_blank = line == "";
    let formated_line = format!(
        "{}{}",
        if is_blank & output_formatter.only_non_blank {
            format!("{:<5}:", String::from(""))
        } else if output_formatter.has_line_numbers {
            format!("{:<5}: ", line_number)
        } else {
            String::from("")
        },
        line
    );
    String::from(formated_line.trim_end())
}

fn main() {
    let output_formatter = read_arguments();

    let file_paths =
        match toolslib::get_file_paths(&output_formatter.inputs, output_formatter.ignore_errors) {
            Ok(file_paths) => file_paths,
            Err(rc) => {
                process::exit(rc as i32);
            }
        };

    // For every file read the contents
    let mut next_line_number = 0u32;
    let stdout = io::stdout();
    let mut handle = io::BufWriter::new(stdout);

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
                    process::exit(toolslib::Rc::ErrorCannotOpenFileForReading as i32);
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

                match writeln!(
                    handle,
                    "{}",
                    format_output_line(&ok_line, next_line_number, &output_formatter)
                ) {
                    Ok(_) => {}
                    Err(err) => {
                        eprintln!("Error {}; when writing to stdout buffer.", err);
                        process::exit(toolslib::Rc::ErrorWriteToStdout as i32);
                    }
                }
            }
            match handle.flush() {
                Ok(_) => {}
                Err(err) => {
                    eprintln!("Error {}; when flushing to stdout.", err);
                    process::exit(toolslib::Rc::ErrorWriteToStdout as i32);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_output_line() {
        let input_string = String::from("my test string");
        let mut output_formatter = OutputFormatter::new();

        // No processing - input matches output
        assert_eq!(
            input_string,
            format_output_line(&input_string, 0, &output_formatter)
        );

        // Add line number
        let string_with_number = String::from("12   : my test string");
        output_formatter.has_line_numbers = true;
        assert_eq!(
            string_with_number,
            format_output_line(&input_string, 12, &output_formatter)
        );

        // Add line number to empty line
        let empty_string_with_number = String::from("13   :");
        output_formatter.has_line_numbers = true;
        assert_eq!(
            empty_string_with_number,
            format_output_line(&String::from(""), 13, &output_formatter)
        );

        // Ignore empty lines
        let empty_string = String::from("");
        let empty_string_no_number = String::from("     :");
        output_formatter.only_non_blank = true;
        output_formatter.has_line_numbers = true;
        assert_eq!(
            empty_string_no_number,
            format_output_line(&empty_string, 14, &output_formatter)
        );
    }
} // mod tests
