///
/// A grep clone written in Rust
/*
 * https://pubs.opengroup.org/onlinepubs/9699919799/utilities/grep.html
 * https://livebook.manning.com/book/rust-in-action/chapter-2/v-16/371
 * https://docs.rs/regex/1.4.5/regex/
 *
 */
use clap::{App, Arg};
use regex::Regex;
use std::env;
use std::ffi::OsString;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::BufReader;
use std::process;

/**
 *
 */

const VERSION: &str = "ver. 0.0.1";

enum GrepError {
    InvalidRegularExpression = 1,
    InputFileNotFound = 2,
}

// A structure that defines how the output is formatted.
struct OutputFormatter {
    ignore_match: bool,
    has_line_numbers: bool,
    with_file_name: bool,
    only_file_names: bool,
    only_line_count: bool,
    pattern: String,
    inputs: Vec<String>,
}

impl OutputFormatter {
    /// Initializes the OutputFormater to default values
    fn new(pattern: &str) -> OutputFormatter {
        OutputFormatter {
            ignore_match: false,
            has_line_numbers: false,
            with_file_name: false,
            only_file_names: false,
            only_line_count: false,
            pattern: String::from(pattern),
            inputs: Vec::new(),
        }
    }
}

/// Read the command line arguments and parse them into the OutputFormatter
/// structure. Return input files in a vector.
fn read_arguments<I, T>(itr: I) -> OutputFormatter
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let matches = App::new("grep: grep clone command written in Rust")
        .version(VERSION)
        .author("Manuel Berrocal")
        .about("searches for patterns in the input text")
        .arg(
            Arg::with_name("line_number")
                .short("n")
                .takes_value(false)
                .help("precede each match with the line number in the file (starting at 1)"),
        )
        .arg(
            Arg::with_name("ignore_match")
                .short("v")
                .takes_value(false)
                .help("select lines not matching the expression"),
        )
        .arg(
            Arg::with_name("with_file_name")
                .short("H")
                .takes_value(false)
                .help("precede each match with input file name"),
        )
        .arg(
            Arg::with_name("only_file_names")
                .short("l")
                .takes_value(false)
                .help("print names of the fileswith content  matching the pattern"),
        )
        .arg(
            Arg::with_name("only_line_count")
                .short("c")
                .takes_value(false)
                .help("print only a count of matching lines to standard output"),
        )
        .arg(
            Arg::with_name("pattern")
                .short("e")
                .help("the pattern to search for")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("inputs")
                .help("Input files, if empty takes input from stdin")
                .required(false)
                .takes_value(true)
                .multiple(true),
        )
        .get_matches_from(itr);

    // unwrap is safe as the pattern argument is required
    let mut output_formatter = OutputFormatter::new(matches.value_of("pattern").unwrap());

    if matches.is_present("ignore_match") {
        output_formatter.ignore_match = true;
    }

    if matches.is_present("line_number") {
        output_formatter.has_line_numbers = true;
    }

    if matches.is_present("with_file_name") {
        output_formatter.with_file_name = true;
    }

    if matches.is_present("only_file_names") {
        output_formatter.only_file_names = true;
    }

    if matches.is_present("only_line_count") {
        output_formatter.only_line_count = true;
    }

    if matches.is_present("inputs") {
        let vals: Vec<&str> = matches.values_of("inputs").unwrap().collect();

        if vals.len() > output_formatter.inputs.capacity() {
            output_formatter.inputs.reserve(vals.len())
        }

        for file_name in vals {
            output_formatter.inputs.push(file_name.to_string())
        }
    } else {
        output_formatter.inputs.push(String::from("-"));
    }

    output_formatter
}

/// Find match in buffer
///
/// Performs a quick match using is_match method for performance.
///
/// # Arguments
/// * `reader` - A `BufRead` containing the text to match.
/// * `re` - A RegEx object containing the regular expression
/// * `ignore_match` - a bool that inverts the matching logic. When `ignore_match`
///    is true returns the files that do not include a match.
///
/// # Return
/// * Return true if buffer content matches the regular expression.
/// * Return false if buffer content matches the regular expression and -v flag.
fn find_match<T: BufRead + Sized>(
    reader: T,
    re: &Regex,
    ignore_match: bool,
) -> Result<bool, GrepError> {
    let found = if ignore_match { false } else { true };
    for line_ in reader.lines() {
        let line = line_.unwrap();
        if re.is_match(line.as_str()) {
            return Ok(found);
        }
    }
    return Ok(!found);
}

/// Returns a vector with the file names matching the regular expression
///
/// # Arguments
/// * `inputs` - A vector of strings containing the path to the files
/// * `re` - The `Regex` object with the regular expression to match
/// * `ignore_match` - a bool that inverts the matching logic.  When `ignore_match`
///    is true returns the files that do not include a match.
///
/// If the standard input is searched, a pathname of "(standard input)" is written.
fn find_matching_files(
    inputs: &Vec<String>,
    re: &Regex,
    ignore_match: bool,
) -> Result<Vec<String>, GrepError> {
    let mut matching_files: Vec<String> = Vec::new();

    for input_file in inputs {
        if input_file == "-" {
            let stdin = io::stdin();
            let reader = stdin.lock();
            match find_match(reader, re, ignore_match) {
                Ok(res) => {
                    if res {
                        matching_files.push(String::from("standard input"));
                    }
                }
                Err(err) => return Err(err),
            }
        } else {
            let f = File::open(input_file).unwrap();
            let reader = BufReader::new(f);
            match find_match(reader, re, ignore_match) {
                Ok(res) => {
                    if res {
                        matching_files.push(String::from(input_file));
                    }
                }
                Err(err) => return Err(err),
            }
        }
    }
    Ok(matching_files)
}

/// Returns the lines in the buffer that match the regular expression.
///
/// # Arguments
/// * `reader` - A `BufRead` containing the text to match.
/// * `re` - A RegEx object containing the regular expression
/// * `ignore_match` - a bool that inverts the matching logic. When `ignore_match`
///    is true returns the files that do not include a match.
///
/// # Returns
/// Returns a vector of tupples,
/// * `line number` : usize
/// * `line text` : String
fn match_lines<T: BufRead + Sized>(
    reader: T,
    re: &Regex,
    ignore_match: bool,
) -> Result<Vec<(usize, String)>, GrepError> {
    let mut matched_lines = Vec::new();
    for (i, line_) in reader.lines().enumerate() {
        let line = line_.unwrap();
        if ignore_match && (!re.is_match(line.as_str())) {
            matched_lines.push((i, line));
        } else if (!ignore_match) && re.is_match(line.as_str()) {
            matched_lines.push((i, line));
        }
    }
    Ok(matched_lines)
}

fn main() {
    let output_formatter = read_arguments(env::args_os());
    let re = match Regex::new(output_formatter.pattern.as_str()) {
        Ok(m) => m,
        Err(_) => {
            eprintln!(
                "Error: {} is not a valid regular expression",
                output_formatter.pattern.as_str()
            );
            process::exit(GrepError::InvalidRegularExpression as i32);
        }
    };

    // Fast implementation for finding files that match the expression
    if output_formatter.only_file_names {
        match find_matching_files(&output_formatter.inputs, &re, output_formatter.ignore_match) {
            Ok(matched_files) => {
                for file_name in matched_files {
                    println!("{}", file_name.as_str());
                }
                return;
            }
            Err(err) => {
                eprintln!("Error");
                process::exit(err as i32);
            }
        }
    }

    // More complex implementation for finding lines that match the expression
    let mut line_count: usize = 0;
    for input_file in &output_formatter.inputs {
        // line number, line
        let mut lines: Vec<(usize, String)> = Vec::new();
        let current_file: String;
        if input_file == "-" {
            current_file = String::from("standard input");
            let stdin = io::stdin();
            let reader = stdin.lock();
            match match_lines(reader, &re, output_formatter.ignore_match) {
                Ok(lines_) => {
                    for line in lines_ {
                        lines.push((line.0, line.1));
                    }
                }
                Err(err) => {
                    eprintln!("Error");
                    process::exit(err as i32);
                }
            }
        } else {
            current_file = input_file.to_string();
            let f = File::open(input_file).unwrap();
            let reader = BufReader::new(f);
            match match_lines(reader, &re, output_formatter.ignore_match) {
                Ok(lines_) => {
                    for line in lines_ {
                        lines.push((line.0, line.1));
                    }
                }
                Err(err) => {
                    eprintln!("Error");
                    process::exit(err as i32);
                }
            }
        }

        line_count += lines.len();
        if output_formatter.only_line_count {
            continue;
        }

        for line in lines {
            println!(
                "{}{}{}",
                if output_formatter.with_file_name {
                    format!("{} ", current_file)
                } else {
                    format!("")
                },
                if output_formatter.has_line_numbers {
                    format!("{}: ", line.0)
                } else {
                    format!("")
                },
                line.1
            );
        }
    }

    if output_formatter.only_line_count {
        println!("{}", line_count);
    }
}
