// Plan 081 Phase 4: Multi-Mode Compilation Pipeline
//
// This module implements compilation of dependencies according to their
// specified execution mode (AutoVM, C transpilation, Rust transpilation).

use crate::error::{AutoError, AutoResult};
use crate::mode::ExecutionMode;
use crate::vm::codegen::Codegen;
use crate::vm::opcode::OpCode;
use std::collections::HashMap;
use std::path::PathBuf;

/// Compilation result for a single dependency
///
/// Contains the compiled output based on the execution mode.
#[derive(Clone)]
pub enum CompiledOutput {
    /// AutoVM bytecode module
    Bytecode {
        bytecode: Vec<u8>,
        bytecode_path: PathBuf,
    },
    /// C transpilation output (.c and .h files)
    C {
        c_file: PathBuf,
        h_file: PathBuf,
    },
    /// Rust transpilation output (.rs file)
    Rust {
        rs_file: PathBuf,
    },
    /// No compilation needed (Evaluator mode)
    Parsed {
        source_path: PathBuf,
    },
}

impl CompiledOutput {
    /// Get the execution mode for this output
    pub fn mode(&self) -> ExecutionMode {
        match self {
            CompiledOutput::Bytecode { .. } => ExecutionMode::AutoVM,
            CompiledOutput::C { .. } => ExecutionMode::C,
            CompiledOutput::Rust { .. } => ExecutionMode::Rust,
            CompiledOutput::Parsed { .. } => ExecutionMode::Evaluator,
        }
    }

    /// Get the main output file path
    pub fn main_file(&self) -> Option<&PathBuf> {
        match self {
            CompiledOutput::Bytecode { bytecode_path, .. } => Some(bytecode_path),
            CompiledOutput::C { c_file, .. } => Some(c_file),
            CompiledOutput::Rust { rs_file } => Some(rs_file),
            CompiledOutput::Parsed { source_path } => Some(source_path),
        }
    }
}

/// Multi-mode compiler
///
/// **Plan 081 Phase 4**: Compiles dependencies according to their execution mode.
///
/// This compiler handles:
/// - AutoVM bytecode compilation
/// - C transpilation (via a2c) - TODO
/// - Rust transpilation (via a2r) - TODO
/// - Parse-only (Evaluator mode)
pub struct MultiModeCompiler {
    /// Output directory for compiled artifacts
    output_dir: PathBuf,
    /// Cache of compiled outputs
    compiled: HashMap<String, CompiledOutput>,
}

impl MultiModeCompiler {
    /// Create a new multi-mode compiler
    ///
    /// # Arguments
    /// * `output_dir` - Directory where compiled outputs will be stored
    pub fn new(output_dir: PathBuf) -> Self {
        Self {
            output_dir,
            compiled: HashMap::new(),
        }
    }

    /// Compile a dependency according to its execution mode
    ///
    /// **Plan 081 Phase 4**: Main entry point for multi-mode compilation
    ///
    /// # Arguments
    /// * `name` - Dependency name
    /// * `source_path` - Path to the source .at file
    /// * `mode` - Execution mode for this dependency
    /// * `source_code` - Optional source code (if already loaded)
    ///
    /// # Returns
    /// * `CompiledOutput` - The compiled artifact
    pub fn compile_dependency(
        &mut self,
        name: &str,
        source_path: PathBuf,
        mode: ExecutionMode,
        source_code: Option<&str>,
    ) -> AutoResult<CompiledOutput> {
        // Load source code if not provided
        let code = if let Some(code) = source_code {
            code.to_string()
        } else {
            std::fs::read_to_string(&source_path)
                .map_err(|e| AutoError::Msg(format!("Failed to read {}: {}", source_path.display(), e)))?
        };

        // Compile according to mode
        let output = match mode {
            ExecutionMode::AutoVM => {
                self.compile_to_bytecode(name, &code)?
            }
            ExecutionMode::C => {
                // TODO: Use trans_c when the API is ready
                return Err(AutoError::Msg("C transpilation not yet implemented in multi-mode compiler".to_string()));
            }
            ExecutionMode::Rust => {
                // TODO: Use trans_rust when the API is ready
                return Err(AutoError::Msg("Rust transpilation not yet implemented in multi-mode compiler".to_string()));
            }
            ExecutionMode::Evaluator => {
                CompiledOutput::Parsed {
                    source_path: source_path.clone(),
                }
            }
        };

        // Cache the output
        self.compiled.insert(name.to_string(), output.clone());

        Ok(output)
    }

    /// Compile source to AutoVM bytecode
    fn compile_to_bytecode(
        &mut self,
        name: &str,
        code: &str,
    ) -> AutoResult<CompiledOutput> {
        use crate::parser::Parser;

        // 1. Parse the code
        let mut parser = Parser::from(code);
        let ast = parser.parse()?;

        // 2. Compile to bytecode
        let mut codegen = Codegen::new();
        for stmt in &ast.stmts {
            codegen.compile_stmt(stmt)?;
        }

        // Add HALT at the end
        codegen.code.push(OpCode::HALT as u8);

        // 3. Perform linking
        for reloc in &codegen.relocs {
            if let Some(&addr) = codegen.exports.get(&reloc.symbol_name) {
                let bytes = addr.to_le_bytes();
                let offset = reloc.offset as usize;
                for (i, b) in bytes.iter().enumerate() {
                    codegen.code[offset + i] = *b;
                }
            } else {
                return Err(AutoError::Msg(format!(
                    "Undefined symbol in {}: {}",
                    name, reloc.symbol_name
                )));
            }
        }

        // 4. Write bytecode to file
        let bytecode_path = self.output_dir.join(format!("{}.bc", name));
        std::fs::write(&bytecode_path, &codegen.code)
            .map_err(|e| AutoError::Msg(format!("Failed to write bytecode: {}", e)))?;

        println!("Compiled {} to AutoVM bytecode: {}", name, bytecode_path.display());

        Ok(CompiledOutput::Bytecode {
            bytecode: codegen.code,
            bytecode_path,
        })
    }

    /// Get compiled output for a dependency
    pub fn get_output(&self, name: &str) -> Option<&CompiledOutput> {
        self.compiled.get(name)
    }

    /// Get all compiled outputs
    pub fn get_all_outputs(&self) -> &HashMap<String, CompiledOutput> {
        &self.compiled
    }

    /// Clear the compilation cache
    pub fn clear_cache(&mut self) {
        self.compiled.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multi_mode_compiler_creation() {
        let compiler = MultiModeCompiler::new(PathBuf::from("target/output"));
        assert_eq!(compiler.output_dir, PathBuf::from("target/output"));
        assert_eq!(compiler.compiled.len(), 0);
    }

    #[test]
    fn test_compiled_output_mode() {
        let bytecode = CompiledOutput::Bytecode {
            bytecode: vec![0x01, 0x02, 0x03],
            bytecode_path: PathBuf::from("test.bc"),
        };
        assert_eq!(bytecode.mode(), ExecutionMode::AutoVM);

        let c_output = CompiledOutput::C {
            c_file: PathBuf::from("test.c"),
            h_file: PathBuf::from("test.h"),
        };
        assert_eq!(c_output.mode(), ExecutionMode::C);

        let rust_output = CompiledOutput::Rust {
            rs_file: PathBuf::from("test.rs"),
        };
        assert_eq!(rust_output.mode(), ExecutionMode::Rust);

        let parsed = CompiledOutput::Parsed {
            source_path: PathBuf::from("test.at"),
        };
        assert_eq!(parsed.mode(), ExecutionMode::Evaluator);
    }

    #[test]
    fn test_compiled_output_main_file() {
        let bytecode = CompiledOutput::Bytecode {
            bytecode: vec![0x01, 0x02, 0x03],
            bytecode_path: PathBuf::from("test.bc"),
        };
        assert_eq!(bytecode.main_file(), Some(&PathBuf::from("test.bc")));

        let c_output = CompiledOutput::C {
            c_file: PathBuf::from("test.c"),
            h_file: PathBuf::from("test.h"),
        };
        assert_eq!(c_output.main_file(), Some(&PathBuf::from("test.c")));
    }

    #[test]
    fn test_clear_cache() {
        let mut compiler = MultiModeCompiler::new(PathBuf::from("target/output"));

        // Add a mock entry
        compiler.compiled.insert(
            "test".to_string(),
            CompiledOutput::Parsed {
                source_path: PathBuf::from("test.at"),
            },
        );

        assert_eq!(compiler.compiled.len(), 1);
        compiler.clear_cache();
        assert_eq!(compiler.compiled.len(), 0);
    }

    #[test]
    fn test_compile_simple_autovm() {
        use std::fs;

        // Create temp directory
        let temp_dir = std::env::temp_dir();
        let output_dir = temp_dir.join("multimode_test");
        fs::create_dir_all(&output_dir).ok();

        let mut compiler = MultiModeCompiler::new(output_dir.clone());

        // Compile simple AutoLang code
        let code = r#"
fn add(a int, b int) int {
    a + b
}

fn main() {
    say(add(1, 2))
}
"#;

        let result = compiler.compile_dependency(
            "test",
            PathBuf::from("test.at"),
            ExecutionMode::AutoVM,
            Some(code)
        );

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(matches!(output, CompiledOutput::Bytecode { .. }));

        // Cleanup
        if let Some(CompiledOutput::Bytecode { bytecode_path, .. }) = compiler.get_output("test") {
            let _ = fs::remove_file(bytecode_path);
        }
        let _ = fs::remove_dir(&output_dir);
    }
}
