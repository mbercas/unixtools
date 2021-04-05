use std::path::Path;

/**
 * Exit codes, note that Process::exit requires i32 as argument
 */
pub enum Rc {
    ErrorInvalidIinputFilePath = 1,
    ErrorCannotOpenFileForReading = 2,
    ErrorWriteToStdout = 3,
}

/**
 * Check that the list of strings passed as an argument describes valid paths.
 *
 * Gets a vector of strings as an input argument and returns an array of valid
 * Paths.
 *
 * If the `ignore_errors` argument is set to true, ignore strings that refer
 * to invalid paths, prints an error message is stderr and continues parsing
 * arguments. If ignore_errors is set to false returns error if any string
 * corresponds to an invalid path.
 */
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
