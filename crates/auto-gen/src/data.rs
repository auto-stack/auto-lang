use crate::error::{GenError, GenResult};
use auto_atom::Atom;
use std::path::PathBuf;

/// Data source for the code generator
#[derive(Clone)]
pub enum DataSource {
    AutoFile(PathBuf),
    AutoCode(String),
    Atom(Atom),
}

/// Loads data from various sources
pub struct DataLoader;

impl DataLoader {
    pub fn new() -> Self {
        Self
    }

    pub fn load(&self, source: DataSource) -> GenResult<Atom> {
        match source {
            DataSource::Atom(atom) => Ok(atom),
            DataSource::AutoFile(path) => {
                let code = std::fs::read_to_string(&path).map_err(|e| GenError::DataLoadError {
                    path: path.clone(),
                    reason: e.to_string(),
                })?;
                self.parse_auto_to_atom(&code, path)
            }
            DataSource::AutoCode(code) => self.parse_auto_to_atom(&code, PathBuf::from("<code>")),
        }
    }

    fn parse_auto_to_atom(&self, code: &str, path: PathBuf) -> GenResult<Atom> {
        // For now, return an empty Atom
        // TODO: Implement proper conversion from Auto code to Atom
        // This requires evaluating the code and extracting variables from the Universe
        let _ = code;
        let _ = path;
        Ok(Atom::default())
    }
}

impl Default for DataLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use auto_val::Value;

    #[test]
    fn test_load_atom() {
        let loader = DataLoader::new();
        let atom = Atom::assemble(vec![Value::pair("test", 123)]);
        let result = loader.load(DataSource::Atom(atom.clone())).unwrap();
        assert_eq!(result.name, atom.name);
    }

    #[test]
    fn test_load_auto_code() {
        let loader = DataLoader::new();
        let code = r#"
let x = 42
let y = "hello"
"#;
        let result = loader.load(DataSource::AutoCode(code.to_string()));
        assert!(result.is_ok());
    }
}
