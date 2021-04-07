/// toolslib
///
/// Library of common functions to the Unixtools
use std::path::Path;

/// Exit codes, note that Process::exit requires i32 as argument
pub enum Rc {
    /// Invalid file path error
    ErrorInvalidIinputFilePath = 1,
    /// The file can not be open for reading
    ErrorCannotOpenFileForReading = 2,
    /// Error writing to standard output
    ErrorWriteToStdout = 3,
}

/// Gets a vector of strings as an input argument and returns an array of valid  Paths.
///
/// # Arguments
///
/// * `inputs` - A vector of strings containing the paths to files
/// * `ignore_errors` - A bool indicating if errors should be ignored
///
/// If the `ignore_errors` argument is set to true, ignore strings that refer
/// to invalid paths, prints an error message is stderr and continues parsing
/// arguments. If `ignore_errors` is set to false returns error if any string
/// corresponds to an invalid path.
pub fn get_file_paths(inputs: &Vec<String>, ignore_errors: bool) -> Result<Vec<&Path>, Rc> {
    let mut file_paths = Vec::new();
    for file_name in inputs {
        let path = Path::new(file_name.as_str());
        if !path.exists() {
            eprintln!("ERROR: file: `{}` does not exist", path.display());
            if !ignore_errors {
                return Err(Rc::ErrorInvalidIinputFilePath);
            }
        }
        file_paths.push(path);
    }
    Ok(file_paths)
}
