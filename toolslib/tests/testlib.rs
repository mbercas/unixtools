mod toolslib {
    extern crate toolslib;
    use toolslib::*;
    #[test]
    fn ts_get_file_paths_invalid_path() {
        let inputs = vec![String::from("invalid_path")];
        let ignore_errors_is_false = false;
        let ignore_errors_is_true = true;

        // test for invalid paths -> return err
        match get_file_paths(&inputs, ignore_errors_is_false) {
            Ok(_) => {}
            Err(err) => assert_eq!(err, ErrCode::ErrorInvalidIinputFilePath),
        }

        // input vector of paths is empty -> return err
        let empty_inputs: Vec<String> = Vec::new();
        match get_file_paths(&empty_inputs, ignore_errors_is_false) {
            Ok(_) => {}
            Err(err) => assert_eq!(err, ErrCode::ErrorInvalidIinputFilePath),
        }

        // test for invalid paths ignore error -> return Ok with empty vector
        match get_file_paths(&inputs, ignore_errors_is_true) {
            Ok(paths) => {
                assert_eq!(0, paths.len())
            }
            Err(_) => assert!(false), // force failure
        }
    }

    #[test]
    fn ts_get_file_paths_stdin() {
        let inputs = vec![String::from("-")];
        let ignore_errors_is_false = false;

        // test for invalid paths -> return err
        match get_file_paths(&inputs, ignore_errors_is_false) {
            Ok(paths) => {
                assert_eq!(1, paths.len());
                assert_eq!("-", paths[0].as_os_str());
            }
            Err(_) => {}
        }
    }
}
