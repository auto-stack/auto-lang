use crate::error::{GenError, GenResult};
use auto_lang::atom::Atom;
use auto_lang::interpreter::AutoInterpreter;
use auto_val::{Shared, Value};
use std::path::PathBuf;

/// Data source for the code generator
#[derive(Clone)]
pub enum DataSource {
    AutoFile(PathBuf),
    AutoCode(String),
    Atom(Atom),
}

/// Loaded data with interpreter context
pub struct LoadedData {
    pub interp: Shared<AutoInterpreter>,
}

/// Loads data from various sources
pub struct DataLoader {
    /// Library search paths for `use` statements (TODO: implement in AutoInterpreter)
    lib_paths: Vec<PathBuf>,
}

impl DataLoader {
    pub fn new() -> Self {
        Self {
            lib_paths: Vec::new(),
        }
    }

    pub fn set_lib_paths(&mut self, paths: Vec<PathBuf>) {
        self.lib_paths = paths;
    }

    /// Add a single library search path for `use` statements
    ///
    /// # Example
    /// ```
    /// use auto_gen::DataLoader;
    /// let mut loader = DataLoader::new();
    /// loader.add_lib_path("./data/libs");
    /// loader.add_lib_path("/usr/local/my_modules");
    /// ```
    pub fn add_lib_path(&mut self, path: impl Into<PathBuf>) {
        self.lib_paths.push(path.into());
    }

    /// Get current library search paths
    pub fn lib_paths(&self) -> &[PathBuf] {
        &self.lib_paths
    }

    pub fn load(&self, source: DataSource) -> GenResult<LoadedData> {
        match source {
            DataSource::Atom(atom) => {
                // Create an interpreter and merge the atom data
                let mut interp = AutoInterpreter::new();
                interp.merge_atom(&atom);
                Ok(LoadedData {
                    interp: auto_val::shared(interp),
                })
            }
            DataSource::AutoFile(path) => {
                let code = std::fs::read_to_string(&path).map_err(|e| GenError::DataLoadError {
                    path: path.clone(),
                    reason: e.to_string(),
                })?;
                self.parse_auto_to_data(&code, path)
            }
            DataSource::AutoCode(code) => self.parse_auto_to_data(&code, PathBuf::from("<code>")),
        }
    }

    fn parse_auto_to_data(&self, code: &str, _path: PathBuf) -> GenResult<LoadedData> {
        eprintln!("DEBUG DataLoader: lib_paths = {:?}", self.lib_paths);
        eprintln!("DEBUG DataLoader: code starts with: {}", &code.chars().take(50).collect::<String>());

        // Evaluate the Auto code
        let mut interp = AutoInterpreter::new();

        // TODO: Add lib_paths support to AutoInterpreter
        // if !self.lib_paths.is_empty() {
        //     interp.set_lib_paths(self.lib_paths.clone());
        // }

        let value = interp.eval(code);

        eprintln!("DEBUG DataLoader: eval result = {:?}", value);

        // Check for errors
        let value = value.map_err(|e| GenError::Other(format!("Eval error: {}", e)))?;

        // Try to convert to Atom if it's a Node or Array
        let atom = match value {
            Value::Node(n) => Atom::new(Value::Node(n))
                .map_err(|e| GenError::Other(format!("Failed to create atom from node: {}", e)))?,
            Value::Array(a) => Atom::new(Value::Array(a))
                .map_err(|e| GenError::Other(format!("Failed to create atom from array: {}", e)))?,
            // For other types, the data should be in the interpreter already
            _ => {
                return Ok(LoadedData {
                    interp: auto_val::shared(interp),
                });
            }
        };

        // Merge the atom data into the interpreter
        interp.merge_atom(&atom);

        Ok(LoadedData {
            interp: auto_val::shared(interp),
        })
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
        let atom = Atom::assemble(vec![Value::pair("test", 123)]).unwrap();
        let result = loader.load(DataSource::Atom(atom));
        assert!(result.is_ok());
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
