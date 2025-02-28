use std::path::Path;
use normalize_path::NormalizePath;
use auto_val::AutoStr;

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

pub trait PathExt {
    fn unified(&self) -> AutoStr;
}

impl PathExt for Path {
    fn unified(&self) -> AutoStr {
        let res = self.normalize().to_string_lossy().replace("\\", "/").into();
        if res == "" { ".".into() } else { res }
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

/// Pretty print an s-expression with proper indentation.
/// 
/// Example:
/// ```text
/// (code
///   (stmt 
///     (pair 
///       (name name) 
///       (str "hello")))
///   (stmt 
///     (pair 
///       (name version)
///       (str "0.1.0"))))
/// ```
pub fn pretty(text: &str) -> String {
    let mut result = String::new();
    let mut indent = 0;
    let mut in_str = false;
    
    for c in text.chars() {
        match c {
            '(' if !in_str => {
                if !result.is_empty() {
                    result.push('\n');
                    result.push_str(&"  ".repeat(indent));
                }
                result.push(c);
                indent += 1;
            }
            ')' if !in_str => {
                indent -= 1;
                result.push(c);
            }
            '"' => {
                in_str = !in_str;
                result.push(c);
            }
            ' ' if !in_str => {
                // result.push('\n');
                // result.push_str(&"  ".repeat(indent));
                result.push(c)
            }
            _ => result.push(c)
        }
    }
    result
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

    #[test]
    fn test_pretty_print_sexp() {
        let text = "(code (stmt (pair (name name) (str \"hello\"))) (stmt (pair (name version) (str \"0.1.0\"))) (stmt (node (name exe) (args (str \"hello\")) (props (pair (name dir) (str \"src\")) (pair (name main) (str \"main.c\")))))";
        let pretty = pretty(text);
        println!("{}", pretty);
    }
}
