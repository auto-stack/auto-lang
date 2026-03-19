use std::fs;
use std::path::Path;

/// Compute BLAKE3 hash of a file's contents
/// Returns the first 64 bits as u64 for compact storage
pub fn hash_file(path: &Path) -> std::io::Result<u64> {
    let content = fs::read(path)?;
    let hash = blake3::hash(&content);
    Ok(u64::from_be_bytes(hash.as_bytes()[0..8].try_into().unwrap()))
}

/// Compute BLAKE3 hash of a string
/// Returns the first 64 bits as u64 for compact storage
pub fn hash_string(content: &str) -> u64 {
    let hash = blake3::hash(content.as_bytes());
    u64::from_be_bytes(hash.as_bytes()[0..8].try_into().unwrap())
}

pub fn split_first(in_string: &str, sep: char) -> (&str, &str) {
    let mut splitter = in_string.splitn(2, sep);
    let first = splitter.next().unwrap();
    let second = splitter.next().unwrap();
    (first, second)
}

pub fn split_last(in_string: &str, sep: char) -> (&str, &str) {
    let mut splitter = in_string.rsplitn(2, sep);
    let last = splitter.next().unwrap();
    let second = match splitter.next() {
        Some(s) => s,
        None => "",
    };
    (second, last)
}

/// Select a port from available ports.
///
/// If `input` is provided, it will be used directly.
/// If there's only one port available, it will be selected automatically.
/// Otherwise, prompts the user to select from available ports.
///
/// # Arguments
/// * `input` - Optional user-specified port name
/// * `available_ports` - List of available port names
/// * `prompt_msg` - Message to display for interactive selection
///
/// # Returns
/// The selected port name
pub fn select_or_default_port(
    input: Option<String>,
    available_ports: &[auto_val::AutoStr],
    prompt_msg: &str,
) -> auto_val::AutoResult<auto_val::AutoStr> {
    use dialoguer::Select;

    let port = if let Some(input) = input {
        input.into()
    } else {
        if available_ports.len() == 1 {
            available_ports[0].clone()
        } else {
            let selection = Select::new()
                .with_prompt(prompt_msg)
                .default(0)
                .items(available_ports)
                .interact()?;

            available_ports[selection].clone()
        }
    };
    Ok(port)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_once() {
        let (first, second) = split_first("assets/templates/lib/mylib.h", '/');
        assert_eq!(first, "assets");
        assert_eq!(second, "templates/lib/mylib.h");
    }

    #[test]
    fn test_split_last() {
        let (first, second) = split_last("assets/templates/lib/mylib.h", '/');
        assert_eq!(first, "assets/templates/lib");
        assert_eq!(second, "mylib.h");
    }
}
