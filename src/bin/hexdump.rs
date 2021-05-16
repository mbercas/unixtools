use clap::{App, Arg};
use std::ffi::OsString;
use std::fs;
///
/// A clone of hexdump
///
use std::{cmp, env};

extern crate toolslib;
use crate::toolslib::ErrCode;

const VERSION: &str = "ver. 0.0.1";

#[derive(Debug)]
struct CommandLineOptions {
    one_byte_octal: bool,
    one_byte_char: bool,
    cannonical: bool,
    two_bytes_hex: bool,
    two_bytes_decimal: bool,
    two_bytes_octal: bool,
    length_bytes: i32,
    offset: i32,
    input_file: String,
}

impl CommandLineOptions {
    fn new() -> CommandLineOptions {
        CommandLineOptions {
            one_byte_octal: false,
            one_byte_char: false,
            cannonical: false,
            two_bytes_hex: true,
            two_bytes_decimal: false,
            two_bytes_octal: false,
            length_bytes: 0,
            offset: 0,
            input_file: String::from(""),
        }
    }
}

fn read_arguments<I, T>(itr: I) -> Result<CommandLineOptions, ErrCode>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let mut cmd_options = CommandLineOptions::new();
    let matches: _ = App::new("hexdump: hexdump clone command written in Rust")
        .version(VERSION)
        .author("Manuel Berrocal")
        .about("Display file contects in hexadecimal, decimal, orctal or ASCII")
        .arg(
            Arg::with_name("one_byte_octal")
                .short("b")
                .long("one-byte-octal")
                .conflicts_with_all(&[
                    "one_byte_char",
                    "cannonical",
                    "two_bytes_hex",
                    "two_bytes_decimal",
                    "two_bytes_octal",
                ])
                .takes_value(false)
                .help("One byte octal display."),
        )
        .arg(
            Arg::with_name("one_byte_char")
                .short("c")
                .long("one-byte-char")
                .takes_value(false)
                .conflicts_with_all(&[
                    "one_byte_octal",
                    "cannonical",
                    "two_bytes_hex",
                    "two_bytes_decimal",
                    "two_bytes_octal",
                ])
                .help("One byte character display."),
        )
        .arg(
            Arg::with_name("cannonical")
                .short("C")
                .long("Cannonical")
                .takes_value(false)
                .conflicts_with_all(&[
                    "one_byte_octal",
                    "one_byte_octal",
                    "two_bytes_hex",
                    "two_bytes_decimal",
                    "two_bytes_octal",
                ])
                .help("Canonical hex+ASCII display."),
        )
        .arg(
            Arg::with_name("two_bytes_hex")
                .short("x")
                .long("two-bytes-hex")
                .takes_value(false)
                .conflicts_with_all(&[
                    "one_byte_char",
                    "one_byte_octal",
                    "cannonical",
                    "two_bytes_decimal",
                    "two_bytes_octal",
                ])
                .help("One byte character display."),
        )
        .arg(
            Arg::with_name("two_bytes_decimal")
                .short("d")
                .long("two-bytes-decimal")
                .takes_value(false)
                .conflicts_with_all(&[
                    "one_byte_char",
                    "one_byte_octal",
                    "cannonical",
                    "two_bytes_hex",
                    "two_bytes_octal",
                ])
                .help("Two bytes decimal display."),
        )
        .arg(
            Arg::with_name("two_bytes_octal")
                .short("o")
                .long("two-bytes-octal")
                .takes_value(false)
                .conflicts_with_all(&[
                    "one_byte_char",
                    "one_byte_octal",
                    "cannonical",
                    "two_bytes_hex",
                    "two_bytes_decimal",
                ])
                .help("Two bytes octal display."),
        )
        .arg(
            Arg::with_name("length")
                .short("n")
                .long("length")
                .takes_value(true)
                .multiple(false)
                .help("Number of input bytes to interpret."),
        )
        .arg(
            Arg::with_name("offset")
                .short("s")
                .long("skip")
                .takes_value(true)
                .multiple(false)
                .help("Skip offset."),
        )
        .arg(
            Arg::with_name("file_name")
                .help("Input file")
                .required(true)
                .takes_value(true)
                .multiple(false),
        )
        .get_matches_from(itr);

    if let Some(i) = matches.value_of("file_name") {
        cmd_options.input_file = String::from(i)
    }

    if let Some(i) = matches.value_of("length") {
        if let Ok(i) = i.parse::<i32>() {
            cmd_options.length_bytes = i.abs()
        } else {
            eprintln!("Length bytes takes only integer arguments");
            return Err(ErrCode::ErrorArgumentParsing);
        }
    }

    if let Some(i) = matches.value_of("offset") {
        match i.parse::<i32>() {
            Ok(i) => cmd_options.offset = i.abs(),
            Err(_) => return Err(ErrCode::ErrorArgumentParsing),
        }
    }

    if matches.is_present("one_byte_octal") {
        cmd_options.two_bytes_hex = false;
        cmd_options.one_byte_octal = true;
    } else if matches.is_present("one_byte_char") {
        cmd_options.two_bytes_hex = false;
        cmd_options.one_byte_char = true;
    } else if matches.is_present("cannonical") {
        cmd_options.two_bytes_hex = false;
        cmd_options.cannonical = true;
    }

    Ok(cmd_options)
}

fn get_input(input_file_name: &String) -> Result<Vec<u8>, ErrCode> {
    match fs::read(input_file_name) {
        Ok(b) => return Ok(b),
        Err(_) => return Err(ErrCode::ErrorArgumentParsing),
    }
}

/**
 Given a buffer and a format implementes an iterator
 that returns formatted strings
*/
#[derive(Debug)]
struct Formatter {
    buf: Vec<u8>,
    cannonical: bool,
    one_byte_output: bool,
    two_byte_output: bool,
    hex_output: bool,
    char_output: bool,
    dec_output: bool,
    oct_output: bool,
    offset: usize,
    bytes_per_line: usize,
}

impl Formatter {
    fn new(buf: Vec<u8>, cmd_options: &CommandLineOptions) -> Formatter {
        let mut fmt = Formatter {
            buf: buf,
            cannonical: false,
            one_byte_output: false,
            two_byte_output: false,
            hex_output: false,
            char_output: false,
            dec_output: false,
            oct_output: false,
            offset: cmd_options.offset as usize,
            bytes_per_line: 16,
        };

        if cmd_options.cannonical {
            fmt.cannonical = true;
        } else if cmd_options.one_byte_char | cmd_options.one_byte_octal {
            fmt.one_byte_output = true;
        } else {
            fmt.two_byte_output = true;
        }

        if (cmd_options.cannonical == false) & cmd_options.two_bytes_decimal {
            fmt.dec_output;
        } else if (cmd_options.cannonical == false)
            & (cmd_options.one_byte_octal | cmd_options.two_bytes_octal)
        {
            fmt.oct_output = true;
        } else if (cmd_options.cannonical == false) & cmd_options.two_bytes_hex {
            fmt.hex_output = true;
        } else if (cmd_options.cannonical == false) & cmd_options.one_byte_char {
            fmt.char_output = true;
        }

        fmt
    }
}


/**
  Returns the string that represents the characted passed as a byte.

  - char_byte (u8): the byte to interpret
  - scape_control_char (bool): scapes control chars with '\' if true.
    Otherwise converts them to '.'
*/
fn get_char_string_rep(char_byte: &[u8], scape_control_char: bool) -> String {
    let c = String::from_utf8_lossy(char_byte);
    if (char_byte[0] as char).is_control() {
        if scape_control_char {
            c.escape_default().to_string()
        } else {
            String::from(".")
        }
    } else {
        c.to_string()
    }

}

impl Iterator for Formatter {
    type Item = String;

    fn next(&mut self) -> Option<String> {
        let mut output: String;

        output = format!("{:07x}", self.offset);
        if self.offset < self.buf.len() {
            let increment = cmp::min(self.buf.len() - self.offset, self.bytes_per_line);
            let end = self.offset + increment;
            let mut ascci_str = String::from("");
            let mut bytes: String = String::from("");

            if self.one_byte_output {
                if self.oct_output {
                    for i in self.offset..end {
                        bytes = format!("{} {:03o}", bytes, self.buf[i]);
                    }
                } else if self.char_output {
                    for i in self.offset..end {
                        // let s = get_char_string_rep(&self.buf[i..(i+1)], true);
                        bytes = format!("{}{:>4}", bytes, get_char_string_rep(&self.buf[i..(i+1)], true));
                    }
                } else {
                    for i in self.offset..end {
                        bytes = format!("{} {:02x}", bytes, self.buf[i]);
                    }
                }
            } else if self.cannonical {
                ascci_str = format!("  |");

                for i in self.offset..end {
                    let extra_space = if i == 8 {
                        " "
                    } else {
                        ""
                    };
                    bytes = format!("{}{} {:02x}", bytes, extra_space, self.buf[i]);
                    ascci_str = format!("{}{}",
                                        ascci_str,
                                        get_char_string_rep(&self.buf[i..(i+1)], false)
                    );
                }
                ascci_str = format!("{}|", ascci_str);
            }
            self.offset += increment;
            output = format!("{} {}", output, bytes);
            if self.cannonical {
                output = format!("{:<57} {}", output, ascci_str);
            }
            Some(output)
        } else {
            None
        }
    }
}

fn main() -> Result<(), ErrCode> {
    let cmd_options = match read_arguments(env::args_os()) {
        Ok(o) => o,
        Err(e) => return Err(e),
    };

    let buf = match get_input(&cmd_options.input_file) {
        Ok(b) => b,
        Err(e) => return Err(e),
    };

    let fmt = Formatter::new(buf, &cmd_options);
    for line in fmt {
        println!("{}", line);
    }

    Ok(())
}

#[cfg(test)]
mod hexdump_ts {
    use super::*;

    #[test]
    fn ts_cmd_line_read_arguments_only_file() {
        // read inputs into vector
        let inputs = vec!["hexdump", "f1"];
        let k_cmd_options = read_arguments(&inputs);
        match k_cmd_options {
            Ok(cmd_options) => {
                assert_eq!("f1", cmd_options.input_file);
                assert_eq!(true, cmd_options.two_bytes_hex);
                assert_eq!(false, cmd_options.one_byte_octal);
                assert_eq!(false, cmd_options.one_byte_char);
                assert_eq!(false, cmd_options.two_bytes_octal);
                assert_eq!(false, cmd_options.two_bytes_decimal);
            }
            Err(_) => assert!(false),
        }
    }

    #[test]
    fn ts_cmd_line_read_arguments_one_byte_octal_arguments() {
        // read inputs into vector
        let inputs = vec!["hexdump", "-b", "f1"];
        let k_cmd_options = read_arguments(&inputs);
        match k_cmd_options {
            Ok(cmd_options) => {
                assert_eq!("f1", cmd_options.input_file);
                assert_eq!(false, cmd_options.two_bytes_hex);
                assert_eq!(true, cmd_options.one_byte_octal);
                assert_eq!(0, cmd_options.length_bytes);
                assert_eq!(0, cmd_options.offset);
            }
            Err(_) => assert!(false),
        }
    }

    #[test]
    fn ts_cmd_line_read_arguments_one_byte_char_arguments() {
        // read inputs into vector
        let inputs = vec!["hexdump", "-c", "f1"];
        let k_cmd_options = read_arguments(&inputs);
        match k_cmd_options {
            Ok(cmd_options) => {
                assert_eq!("f1", cmd_options.input_file);
                assert_eq!(false, cmd_options.two_bytes_hex);
                assert_eq!(true, cmd_options.one_byte_char);
                assert_eq!(0, cmd_options.length_bytes);
                assert_eq!(0, cmd_options.offset);
            }
            Err(_) => assert!(false),
        }
    }

    #[test]
    fn ts_cmd_line_read_arguments_cannonical_arguments() {
        // read inputs into vector
        let inputs = vec!["hexdump", "-C", "f1"];
        let k_cmd_options = read_arguments(&inputs);
        match k_cmd_options {
            Ok(cmd_options) => {
                assert_eq!("f1", cmd_options.input_file);
                assert_eq!(false, cmd_options.two_bytes_hex);
                assert_eq!(true, cmd_options.cannonical);
                assert_eq!(0, cmd_options.length_bytes);
                assert_eq!(0, cmd_options.offset);
            }
            Err(_) => assert!(false),
        }
    }

    #[test]
    fn ts_cmd_line_read_arguments_numerical_arguments() {
        // read inputs into vector
        let inputs = vec!["hexdump", "-n", "10", "-s", "11", "f1"];
        let k_cmd_options = read_arguments(&inputs);
        match k_cmd_options {
            Ok(cmd_options) => {
                assert_eq!("f1", cmd_options.input_file);
                assert_eq!(10, cmd_options.length_bytes);
                assert_eq!(11, cmd_options.offset);
            }
            Err(_) => assert!(false),
        }
    }

    #[test]
    fn ts_formatter_new_octal() {
        let v: Vec<u8> = vec![1, 2, 3];
        let mut cmd_options = CommandLineOptions::new();
        cmd_options.one_byte_octal = true;
        cmd_options.two_bytes_hex = false;
        let fmt: Formatter = Formatter::new(v, &cmd_options);
        assert_eq!(true, fmt.oct_output);
        assert_eq!(false, fmt.char_output);
        assert_eq!(false, fmt.cannonical);
        assert_eq!(true, fmt.one_byte_output);
    }

    #[test]
    fn ts_formatter_one_byte_octal() {
        let buf: Vec<u8> = vec![1, 2, 3, 4, 5, 6, 7, 8, 9];
        let mut cmd_options = CommandLineOptions::new();
        cmd_options.one_byte_octal = true;
        cmd_options.two_bytes_hex = false;
        let fmt = Formatter::new(buf, &cmd_options);

        let mut expected_lines: Vec<String> = Vec::new();
        expected_lines.push(String::from(
            "0000000  001 002 003 004 005 006 007 010 011",
        ));
        expected_lines.push(String::from("00000009"));

        for (i, line) in fmt.enumerate() {
            assert_eq!(expected_lines[i], line, "line is: {}", line);
        }

        let buf: Vec<u8> = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
        let _ = expected_lines.pop();
        let _ = expected_lines.pop();

        expected_lines.push(String::from(
            "0000000  001 002 003 004 005 006 007 010 011 012 013 014 015 016 017 020",
        ));
        expected_lines.push(String::from("0000010"));

        let fmt = Formatter::new(buf, &cmd_options);
        for (i, line) in fmt.enumerate() {
            assert_eq!(expected_lines[i], line, "line is: {}", line);
        }

        let buf: Vec<u8> = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17];
        let _ = expected_lines.pop();

        expected_lines.push(String::from("0000010  021"));
        expected_lines.push(String::from("0000011"));

        let fmt = Formatter::new(buf, &cmd_options);
        for (i, line) in fmt.enumerate() {
            assert_eq!(expected_lines[i], line, "line is: {}", line);
        }
    }

    #[test]
    fn ts_formatter_one_char() {
        let buf: Vec<u8> = vec![66, 67, 68, 69, 70, 71, 72, 73, 74, 75];
        let mut cmd_options = CommandLineOptions::new();
        cmd_options.one_byte_char = true;
        cmd_options.two_bytes_hex = false;
        let fmt = Formatter::new(buf, &cmd_options);

        let mut expected_lines: Vec<String> = Vec::new();
        expected_lines.push(String::from(
            "0000000    B   C   D   E   F   G   H   I   J   K",
        ));
        expected_lines.push(String::from("00000009"));

        for (i, line) in fmt.enumerate() {
            assert_eq!(expected_lines[i], line, "line is: {}", line);
        }
    }

    #[test]
    fn ts_formatter_cannonical() {
        let buf: Vec<u8> = vec![66, 67, 68, 69, 70, 71, 72, 73, 74, 75];
        let mut cmd_options = CommandLineOptions::new();
        cmd_options.cannonical = true;
        cmd_options.two_bytes_hex = false;
        let fmt = Formatter::new(buf, &cmd_options);
        let mut expected_lines: Vec<String> = Vec::new();


        // test one incomplete line
        expected_lines.push(String::from(
            format!("{:<57}   {}", "0000000  42 43 44 45 46 47 48 49  4a 4b", "|BCDEFGHIJK|")
        ));

        for (i, line) in fmt.enumerate() {
            assert_eq!(expected_lines[i], line, "line is: {}", line);
        }

        let buf: Vec<u8> = vec![66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81];
        let fmt = Formatter::new(buf, &cmd_options);

        // test one complete line
        let _ = expected_lines.pop();
        expected_lines.push(String::from(
            format!("{:<57}   {}", "0000000  42 43 44 45 46 47 48 49  4a 4b 4c 4d 4e 4f 50 51", "|BCDEFGHIJKLMNOPQ|")
        ));
        for (i, line) in fmt.enumerate() {
            assert_eq!(expected_lines[i], line, "line is: {}", line);
        }

        // test 2 lines - second incomplete and ends in \n
        let buf: Vec<u8> = vec![66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 0x0a];
        let fmt = Formatter::new(buf, &cmd_options);

        expected_lines.push(String::from(
            format!("{:<57}   {}", "0000010  52 53 0a", "|RS.|")
        ));
        for (i, line) in fmt.enumerate() {
            assert_eq!(expected_lines[i], line, "line is: {}", line);
        }

    }

} // mod hexdump_ts
