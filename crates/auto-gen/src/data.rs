use crate::error::{GenError, GenResult};
use auto_lang::atom::Atom;
use auto_lang::interp::Interpreter;
use auto_lang::Universe;
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
    pub scope: Shared<Universe>,
}

/// Loads data from various sources
pub struct DataLoader {
    /// Library search paths for `use` statements
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
                // Create a universe and merge the atom data
                let mut universe = Universe::new();
                universe.merge_atom(&atom);
                Ok(LoadedData {
                    scope: auto_val::shared(universe),
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
        let mut inter = Interpreter::new();

        // Set library search paths for `use` statements
        if !self.lib_paths.is_empty() {
            inter.set_lib_paths(self.lib_paths.clone());
        }

        let value = inter.eval(code);

        eprintln!("DEBUG DataLoader: eval result is_error = {}", value.is_error());

        // Try to convert to Atom if it's a Node or Array
        let atom = match value {
            Value::Node(n) => Atom::new(Value::Node(n))
                .map_err(|e| GenError::Other(format!("Failed to create atom from node: {}", e)))?,
            Value::Array(a) => Atom::new(Value::Array(a))
                .map_err(|e| GenError::Other(format!("Failed to create atom from array: {}", e)))?,
            // For other types, the data should be in the scope already
            _ => {
                return Ok(LoadedData {
                    scope: inter.scope.clone(),
                });
            }
        };

        // Create a universe and merge the atom data
        let mut universe = Universe::new();
        universe.merge_atom(&atom);

        Ok(LoadedData {
            scope: auto_val::shared(universe),
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
