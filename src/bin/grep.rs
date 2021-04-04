use clap::{App, Arg};
use regex::Regex;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::BufReader;
use std::process;

/**
 *  A grep clone written in Rust
 *
 * https://pubs.opengroup.org/onlinepubs/9699919799/utilities/grep.html
 * https://livebook.manning.com/book/rust-in-action/chapter-2/v-16/371
 * https://docs.rs/regex/1.4.5/regex/
 *
 */

/**
 * A structure that defines how the output is formatted.
 */

const VERSION: &str = "ver. 0.0.1";

enum GrepError {
    InvalidRegularExpression = 1,
    InputFileNotFound = 2,
}

//#[derive(Clone, Copy)]
struct OutputFormatter {
    has_line_numbers: bool,
    only_file_names: bool,
    use_stdin: bool,
    pattern: String,
}

/**
 * Read the command line arguments and parse them into the OutputFormatter
 * structure. Return input files in a vector.
 */
fn read_arguments() -> (OutputFormatter, Vec<String>) {
    let mut output_formatter = OutputFormatter {
        has_line_numbers: false,
        only_file_names: false,
        use_stdin: false,
        pattern: String::from(""),
    };

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
            Arg::with_name("only_file_names")
                .short("l")
                .takes_value(false)
                .help("print names of the fileswith content  matching the pattern"),
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
        .get_matches();

    if matches.is_present("number") {
        output_formatter.has_line_numbers = true;
    }

    if matches.is_present("only_file_names") {
        output_formatter.only_file_names = true;
    }

    // unwrap is safe as the pattern argument is required
    output_formatter.pattern = String::from(matches.value_of("pattern").unwrap());

    let mut inputs = Vec::with_capacity(1);

    if matches.is_present("inputs") {
        let vals: Vec<&str> = matches.values_of("inputs").unwrap().collect();

        if vals.len() > inputs.capacity() {
            inputs.reserve(vals.len())
        }

        for file_name in vals {
            inputs.push(file_name.to_string())
        }
    } else {
        inputs.push(String::from("-"));
    }

    (output_formatter, inputs)
}

/**
 * Find match in file
 *
 * Return true if buffer content matches the regular expression.
 *
 * Performs a quick match using is_match method for performance.
 */
fn find_match<T: BufRead + Sized>(reader: T, re: &Regex) -> Result<bool, GrepError> {
    for line_ in reader.lines() {
        let line = line_.unwrap();
        if re.is_match(line.as_str()) {
            return Ok(true);
        }
    }
    return Ok(false);
}

/**
 * Finds the files in the input list matching the regex.
 *
 * If the standard input is searched, a pathname of "(standard input)" is written
 */
fn find_matching_files(inputs: &Vec<String>, re: &Regex) -> Result<Vec<String>, GrepError> {
    let mut matching_files: Vec<String> = Vec::new();

    for input_file in inputs {
        if input_file == "-" {
            let stdin = io::stdin();
            let reader = stdin.lock();
            match find_match(reader, re) {
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
            match find_match(reader, re) {
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

/**
 * Gets a line buffer and a regular expression and
 * returns prints the lines in the buffer that match the
 * regular expression.
 */
fn process_lines<T: BufRead + Sized>(reader: T, re: &Regex) {
    for line_ in reader.lines() {
        let line = line_.unwrap();
        if re.is_match(line.as_str()) {
            println!("{}", line)
        }
    }
}

fn main() {
    let (output_formatter, inputs) = read_arguments();
    let re = Regex::new(output_formatter.pattern.as_str()).unwrap();

    if output_formatter.only_file_names {
        match find_matching_files(&inputs, &re) {
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

    for input_file in inputs {
        if input_file == "-" {
            let stdin = io::stdin();
            let reader = stdin.lock();
            process_lines(reader, &re);
        } else {
            let f = File::open(input_file).unwrap();
            let reader = BufReader::new(f);
            process_lines(reader, &re);
        }
    }

    let context_lines = 2;
    let haystack = "\
Every face, every shop,
bedroom window, public-house, and
dark square is a picture
feverishly turned--in search of what?
It is the same with books.
What do we seek
through millions of pages?";

    let mut tags: Vec<usize> = Vec::new();
    let mut ctx: Vec<Vec<(usize, String)>> = Vec::new();

    for (i, line) in haystack.lines().enumerate() {
        if re.is_match(line) {
            tags.push(i);
        }

        let v = Vec::with_capacity(2 * context_lines + 1);
        ctx.push(v);
    }

    if tags.is_empty() {
        return;
    }

    for (i, line) in haystack.lines().enumerate() {
        for (j, tag) in tags.iter().enumerate() {
            let lower_bound = tag.saturating_sub(context_lines);
            let upper_bound = tag + context_lines;

            if (i >= lower_bound) && (i <= upper_bound) {
                let line_as_string = String::from(line);
                let local_ctx = (i, line_as_string);
                ctx[j].push(local_ctx);
            }
        }
    }

    for local_ctx in ctx.iter() {
        for &(i, ref line) in local_ctx.iter() {
            let line_num = i + 1;
            println!("{}: {}", line_num, line);
        }
    }
}
