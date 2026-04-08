// Plan 075 Phase 1: ConfigCodegen Implementation
// Compiles config files to bytecode that builds nested object structures

use crate::ast::{Code, Stmt, Store, Expr};
use crate::vm::codegen::Codegen;
use crate::vm::opcode::OpCode;
use crate::vm::loader::Module;
use crate::error::{AutoResult, AutoError};
use auto_val::ValueKey;

/// ConfigCodegen transforms configuration files into bytecode that builds
/// a unified object structure.
///
/// Input (config.at):
/// ```auto
/// server: { host: "localhost", port: 8080 }
/// database: { name: "mydb" }
/// debug: true
/// ```
///
/// Output: bytecode that creates a single object with all fields:
/// ```text
/// LOAD_STR "localhost"
/// LOAD_CONST 8080
/// LOAD_STR "mydb"
/// CONST_1  // true
/// CREATE_OBJ keys=["server", "database", "debug"]
/// RET
/// ```
pub struct ConfigCodegen {
    /// Base codegen for opcode emission
    base: Codegen,
    /// Collected field paths (e.g., ["server.host", "debug"])
    field_paths: Vec<String>,
    /// Collected field values (expressions to compile)
    field_values: Vec<Expr>,
}

impl ConfigCodegen {
    /// Create a new ConfigCodegen instance
    pub fn new() -> Self {
        Self {
            base: Codegen::new(),
            field_paths: Vec::new(),
            field_values: Vec::new(),
        }
    }

    /// Compile config file to bytecode
    ///
    /// Collects all field assignments and creates a single object.
    pub fn compile_config(&mut self, code: &Code) -> AutoResult<()> {
        // Phase 1: Collect all field assignments
        for stmt in &code.stmts {
            self.collect_config_stmt(stmt)?;
        }

        // Phase 2: Compile field values (in normal order so they're pushed correctly)
        for expr in self.field_values.iter() {
            self.base.compile_expr(expr)?;
        }

        // Phase 3: Create object with all fields
        if !self.field_paths.is_empty() {
            self.create_config_object()?;
        }

        // Return the config object
        // RET instruction: opcode (1 byte) + n_args (1 byte)
        self.base.code.push(OpCode::RET as u8);
        self.base.code.push(0); // n_args = 0 for config return

        Ok(())
    }

    /// Collect field assignments from statements
    fn collect_config_stmt(&mut self, stmt: &Stmt) -> AutoResult<()> {
        match stmt {
            // Ignore empty lines
            Stmt::EmptyLine(_) => {
                // Do nothing
            }
            // Parse field assignments: server.host = "localhost"
            Stmt::Store(store) => {
                self.collect_store_field(store)?;
            }
            // Evaluate expressions and add to config
            Stmt::Expr(expr) => {
                self.collect_expr_field(expr)?;
            }
            // Node statements (like app("name") {...}) are treated as expressions
            Stmt::Node(node) => {
                // Convert Node to Expr::Node and collect it
                let node_expr = crate::ast::Expr::Node(node.clone());
                self.collect_expr_field(&node_expr)?;
            }
            _ => {
                return Err(AutoError::Msg(
                    format!("Config mode does not support statement: {:?}", stmt)
                ));
            }
        }
        Ok(())
    }

    /// Collect a store statement as a field assignment
    fn collect_store_field(&mut self, store: &Store) -> AutoResult<()> {
        // Use the full dotted name as the field path
        // e.g., "server.host" stays as "server.host"
        let field_path = store.name.to_string();

        // Clone the expression for later compilation
        let expr = store.expr.clone();

        // Track this field
        self.field_paths.push(field_path);
        self.field_values.push(expr);

        Ok(())
    }

    /// Collect an expression statement as an anonymous field (or named if Pair)
    fn collect_expr_field(&mut self, expr: &Expr) -> AutoResult<()> {
        let (field_name, expr) = if let Expr::Pair(pair) = expr {
            // Unpack pair: key: value -> map key to field_name
            let key_str = match &pair.key {
                crate::ast::Key::NamedKey(name) => name.to_string(),
                crate::ast::Key::StrKey(s) => s.to_string(),
                _ => format!("_expr{}", self.field_values.len()), // Fallback
            };
            (key_str, *pair.value.clone())
        } else {
            // Generate anonymous field name for other expressions
            (format!("_expr{}", self.field_values.len()), expr.clone())
        };

        // Track this field
        self.field_paths.push(field_name);
        self.field_values.push(expr);

        Ok(())
    }

    /// Create the config object with all collected fields
    fn create_config_object(&mut self) -> AutoResult<()> {
        // Register keys in object_keys pool
        let keys: Vec<ValueKey> = self.field_paths
            .iter()
            .map(|s| ValueKey::Str(s.clone().into()))
            .collect();

        let key_index = self.base.object_keys.len() as u16;
        self.base.object_keys.push(keys);

        // Plan 073: Infer field types from field values
        let types: Vec<crate::vm::codegen::ObjectType> = self.field_values.iter()
            .map(|expr| self.base.infer_object_type(expr))
            .collect();
        self.base.object_types.push(types);

        // Emit CREATE_OBJ with key_index and field count
        let field_count = self.field_paths.len() as u8;
        self.base.code.push(OpCode::CREATE_OBJ as u8);
        self.base.code.extend_from_slice(&key_index.to_le_bytes());
        self.base.code.push(field_count);

        Ok(())
    }

    /// Finish compilation and return the module
    pub fn finish(self, name: String) -> Module {
        self.base.finish(name)
    }

    /// Get the base codegen for advanced usage
    pub fn base(&mut self) -> &mut Codegen {
        &mut self.base
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;

    fn parse_source(source: &str) -> Code {
        let mut parser = Parser::from(source);
        parser.parse().unwrap()
    }

    #[test]
    fn test_config_codegen_simple_fields() {
        // Auto Config uses colon syntax (JSON/Atom style)
        let source = r#"
host: "localhost"
port: 8080
debug: true
"#;

        let code = parse_source(source);
        let mut configgen = ConfigCodegen::new();
        configgen.compile_config(&code).unwrap();

        let module = configgen.finish("test".to_string());

        // Verify bytecode contains expected opcodes
        let bytecode = &module.code;
        assert!(bytecode.contains(&0x2E), "Expected CREATE_OBJ opcode (0x2E)");

        // Should have one CREATE_OBJ call with 3 fields
        let create_obj_count = bytecode.iter().filter(|&&x| x == 0x2E).count();
        assert_eq!(create_obj_count, 1, "Expected 1 CREATE_OBJ opcode");

        // Check field count (should be 3)
        if let Some(idx) = bytecode.iter().position(|&x| x == 0x2E) {
            let field_count = bytecode[idx + 3]; // +3 for opcode + 2-byte index
            assert_eq!(field_count, 3, "Expected 3 fields in object");
        }
    }

    #[test]
    fn test_config_codegen_nested_fields() {
        // Auto Config: nested objects use { } blocks with colon syntax
        let source = r#"
server: { host: "localhost", port: 5432 }
database: { name: "mydb" }
"#;

        let code = parse_source(source);
        let mut configgen = ConfigCodegen::new();
        configgen.compile_config(&code).unwrap();

        let module = configgen.finish("test".to_string());

        // Verify bytecode was generated
        let bytecode = &module.code;
        assert!(bytecode.contains(&0x2E), "Expected CREATE_OBJ opcode (0x2E)");

        // Should have at least one CREATE_OBJ for the top-level config
        let create_obj_count = bytecode.iter().filter(|&&x| x == 0x2E).count();
        assert!(create_obj_count >= 1, "Expected at least 1 CREATE_OBJ opcode");
    }

    #[test]
    fn test_config_codegen_with_expressions() {
        // Auto Config: fields use colon syntax
        let source = r#"
max_connections: 10
timeout: 30
"#;

        let code = parse_source(source);
        let mut configgen = ConfigCodegen::new();
        configgen.compile_config(&code).unwrap();

        let module = configgen.finish("test".to_string());

        // Verify bytecode was generated
        let bytecode = &module.code;
        assert!(bytecode.contains(&0x2E), "Expected CREATE_OBJ opcode (0x2E)");

        // Should have one CREATE_OBJ call with 2 fields
        let create_obj_count = bytecode.iter().filter(|&&x| x == 0x2E).count();
        assert_eq!(create_obj_count, 1, "Expected 1 CREATE_OBJ opcode");
    }

    #[test]
    fn test_config_codegen_empty_config() {
        let source = "";

        let mut parser = Parser::from(source);
        let code = parser.parse().unwrap();

        let mut configgen = ConfigCodegen::new();
        configgen.compile_config(&code).unwrap();

        let module = configgen.finish("test".to_string());

        // Should have RET opcode but no CREATE_OBJ
        let bytecode = &module.code;
        assert!(bytecode.contains(&0x71), "Expected RET opcode (0x71)");
        assert!(!bytecode.contains(&0x2E), "Should not have CREATE_OBJ for empty config");
    }
}
