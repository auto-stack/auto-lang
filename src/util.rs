/// Get the file name from a path.
pub fn file_name(path: &str) -> &str {
    if let Some(pos) = path.rfind('/') {
        &path[pos + 1..]
    } else {
        if let Some(pos) = path.rfind('\\') {
            &path[pos + 1..]
        } else {
            path
        }
    }
}

/// Generate an error message with the file name and line number.
#[macro_export]
macro_rules! error_pos {
    ($msg: literal) => {
        Err(format!("{} at {}:{}", $msg, crate::util::file_name(file!()), line!()))
    };
    ($msg: literal, $($args:tt)*) => {
        Err(format!("{} at {}:{}", format!($msg, $($args)*), crate::util::file_name(file!()), line!()))
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_name() {
        assert_eq!(file_name("src/util.rs"), "util.rs");
        assert_eq!(file_name("src/util.rs/"), "");
        assert_eq!(file_name("src\\util.rs"), "util.rs");
        assert_eq!(file_name("src\\util.rs\\"), "");
    }

    #[test]
    fn test_error() {
        let err: Result<(), String> = error_pos!("test error");
        let err_line = line!()-1;
        assert_eq!(format!("{}", err.unwrap_err()), format!("test error at util.rs:{}", err_line));

        let err: Result<(), String> = error_pos!("int error {}", "-1");
        let err_line = line!()-1;
        assert_eq!(format!("{}", err.unwrap_err()), format!("int error -1 at util.rs:{}", err_line));
    }
}
