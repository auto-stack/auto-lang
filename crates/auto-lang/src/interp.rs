use crate::eval::{EvalMode, Evaler};
use crate::parser::Parser;
use crate::universe::Universe;
use crate::AutoResult;
use auto_val::{shared, Shared};
use auto_val::{AutoStr, Value};

pub struct Importer {
    pub path: AutoStr,
    pub scope: Shared<Universe>,
}

pub struct Interpreter {
    pub evaler: Evaler,
    pub scope: Shared<Universe>,
    pub result: Value,
    pub fstr_note: char,
    skip_check: bool,
    /// Enable error recovery to collect multiple errors
    enable_error_recovery: bool,
}

impl Interpreter {
    pub fn new() -> Self {
        let scope = shared(Universe::new());
        let mut interpreter = Self {
            evaler: Evaler::new(scope.clone()),
            scope,
            result: Value::Nil,
            fstr_note: '$',
            skip_check: false,
            enable_error_recovery: false,
        };

        // Register the evaluator with the universe so VM functions can call back
        interpreter.evaler.register_with_universe();

        // Inject environment variables based on target platform (Plan 055)
        // This must happen BEFORE loading Prelude so that Prelude can access env vars
        let target = crate::target::Target::detect();
        interpreter.scope.borrow_mut().inject_environment(target);

        // Initialize VM modules
        crate::vm::init_io_module();
        crate::vm::init_collections_module();
        crate::vm::init_builder_module();
        crate::vm::init_storage_module();

        // Load standard type definitions to register HashMap, HashSet, StringBuilder, List types
        Self::load_stdlib_types(&interpreter.scope);

        // Load dstr.at to register dstr type and methods
        let dstr_code = std::fs::read_to_string("../../stdlib/auto/dstr.at").unwrap_or_else(|_| {
            // Try alternate path
            std::fs::read_to_string("stdlib/auto/dstr.at").unwrap_or(String::new())
        });
        if !dstr_code.is_empty() {
            let _ = interpreter.interpret(&dstr_code);
        }

        // Load list_node.at to register ListNode type and methods
        let list_node_code = std::fs::read_to_string("../../stdlib/auto/list_node.at").unwrap_or_else(|_| {
            // Try alternate path
            std::fs::read_to_string("stdlib/auto/list_node.at").unwrap_or(String::new())
        });
        if !list_node_code.is_empty() {
            let _ = interpreter.interpret(&list_node_code);
        }

        // Load storage.at to register Storage types (Plan 055)
        // This must be loaded BEFORE prelude.at so that prelude can re-export Storage types
        let storage_code = std::fs::read_to_string("../../stdlib/auto/storage.at").unwrap_or_else(|_| {
            // Try alternate path
            std::fs::read_to_string("stdlib/auto/storage.at").unwrap_or(String::new())
        });
        if !storage_code.is_empty() {
            let _ = interpreter.interpret(&storage_code);
        }

        // Load prelude.at to import say and other ubiquitous symbols
        // This is automatically loaded for every program (Plan 050: Auto Prelude System)
        let prelude_code = std::fs::read_to_string("../../stdlib/auto/prelude.at").unwrap_or_else(|_| {
            // Try alternate path
            std::fs::read_to_string("stdlib/auto/prelude.at").unwrap_or(String::new())
        });
        if !prelude_code.is_empty() {
            let _result = interpreter.interpret(&prelude_code);
        }

        // Plan 019 Stage 8.5: Load iter/spec.at to register Iterator specs
        // This is needed because use statements in prelude only load VM modules, not .at spec files
        let iter_spec_code = std::fs::read_to_string("../../stdlib/auto/iter/spec.at").unwrap_or_else(|_| {
            // Try alternate path
            std::fs::read_to_string("stdlib/auto/iter/spec.at").unwrap_or(String::new())
        });
        if !iter_spec_code.is_empty() {
            let _result = interpreter.interpret(&iter_spec_code);
        }

        interpreter
    }

    /// Load standard library type definitions (HashMap, HashSet, StringBuilder)
    /// This makes types like HashMap available for static method calls (HashMap.new())
    fn load_stdlib_types(scope: &Shared<Universe>) {
        use crate::ast::{Name, Type, TypeDecl, TypeDeclKind};

        // Register HashMap type
        let hashmap_type = TypeDecl {
            name: Name::from("HashMap"),
            kind: TypeDeclKind::UserType,
            parent: None,
            has: Vec::new(),
            specs: Vec::new(),
            spec_impls: Vec::new(), // Plan 057
            generic_params: Vec::new(),
            members: Vec::new(),
            delegations: Vec::new(),
            methods: Vec::new(),  // Methods are registered separately via VM registry
        };
        scope.borrow_mut().define_type(
            "HashMap",
            std::rc::Rc::new(crate::scope::Meta::Type(Type::User(hashmap_type))),
        );

        // Register HashSet type
        let hashset_type = TypeDecl {
            name: Name::from("HashSet"),
            kind: TypeDeclKind::UserType,
            parent: None,
            has: Vec::new(),
            specs: Vec::new(),
            spec_impls: Vec::new(), // Plan 057
            generic_params: Vec::new(),
            members: Vec::new(),
            delegations: Vec::new(),
            methods: Vec::new(),
        };
        scope.borrow_mut().define_type(
            "HashSet",
            std::rc::Rc::new(crate::scope::Meta::Type(Type::User(hashset_type))),
        );

        // Register StringBuilder type
        let builder_type = TypeDecl {
            name: Name::from("StringBuilder"),
            kind: TypeDeclKind::UserType,
            parent: None,
            has: Vec::new(),
            specs: Vec::new(),
            spec_impls: Vec::new(), // Plan 057
            generic_params: Vec::new(),
            members: Vec::new(),
            delegations: Vec::new(),
            methods: Vec::new(),
        };
        scope.borrow_mut().define_type(
            "StringBuilder",
            std::rc::Rc::new(crate::scope::Meta::Type(Type::User(builder_type))),
        );

        // Register File type
        let file_type = TypeDecl {
            name: Name::from("File"),
            kind: TypeDeclKind::UserType,
            parent: None,
            has: Vec::new(),
            specs: Vec::new(),
            spec_impls: Vec::new(), // Plan 057
            generic_params: Vec::new(),
            members: Vec::new(),
            delegations: Vec::new(),
            methods: Vec::new(),  // Methods are registered separately via VM registry
        };
        scope.borrow_mut().define_type(
            "File",
            std::rc::Rc::new(crate::scope::Meta::Type(Type::User(file_type))),
        );

        // Register List type
        let list_type = TypeDecl {
            name: Name::from("List"),
            kind: TypeDeclKind::UserType,
            parent: None,
            has: Vec::new(),
            specs: Vec::new(),
            spec_impls: Vec::new(), // Plan 057
            generic_params: Vec::new(),
            members: Vec::new(),
            delegations: Vec::new(),
            methods: Vec::new(),  // Methods are registered separately via VM registry
        };
        scope.borrow_mut().define_type(
            "List",
            std::rc::Rc::new(crate::scope::Meta::Type(Type::User(list_type))),
        );
    }

    pub fn with_fstr_note(mut self, note: char) -> Self {
        self.fstr_note = note;
        self
    }

    pub fn with_univ(univ: Shared<Universe>) -> Self {
        let mut interp = Self {
            evaler: Evaler::new(univ.clone()),
            scope: univ.clone(),
            fstr_note: '$',
            result: Value::Nil,
            skip_check: false,
            enable_error_recovery: false,
        };

        // Register the evaluator with the universe so VM functions can call back
        interp.evaler.register_with_universe();

        // Load standard type definitions to register HashMap, HashSet, StringBuilder, List types
        // This must be called for each new Universe/scope
        Self::load_stdlib_types(&interp.scope);

        interp
    }

    pub fn with_scope(scope: Universe) -> Self {
        Self::with_univ(shared(scope))
    }

    pub fn with_eval_mode(mut self, mode: EvalMode) -> Self {
        self.evaler = self.evaler.with_mode(mode);
        self
    }

    pub fn import(&mut self, _path: AutoStr) -> Result<(), String> {
        // println!("import: {}", path); // LSP: disabled
        Ok(())
    }

    pub fn skip_check(&mut self) {
        self.skip_check = true;
        self.evaler.skip_check();
    }

    /// Enable error recovery to collect multiple errors during parsing
    ///
    /// When enabled, the parser will attempt to recover from syntax errors
    /// and continue parsing to collect additional errors instead of aborting
    /// on the first error.
    pub fn enable_error_recovery(&mut self) {
        self.enable_error_recovery = true;
    }

    pub fn interpret(&mut self, code: &str) -> AutoResult<()> {
        // Create a lexer first to check for errors before creating parser
        let mut lexer = crate::lexer::Lexer::new(code);
        lexer.set_fstr_note(self.fstr_note);
        // Try to get the first token - if it's an error, return it
        let first_token = match lexer.next() {
            Ok(token) => token,
            Err(e) => return Err(e),
        };

        // Create parser with the already-lexed first token
        let mut parser = Parser::new_with_note_and_first_token(
            code,
            self.scope.clone(),
            self.fstr_note,
            first_token,
            lexer,
        );
        if self.skip_check {
            parser = parser.skip_check();
        }
        let ast = parser.parse()?;
        let result = self.evaler.eval(&ast)?;
        // Check if result is an error and return it as a Result error
        if result.is_error() {
            return Err(format!("Evaluation error: {}", result).into());
        }
        let derefed = self.scope.borrow().deref_val(result);
        self.result = derefed;
        Ok(())
    }

    pub fn eval_template(&mut self, code: impl Into<AutoStr>) -> AutoResult<Value> {
        self.eval_template_with_note(code, self.fstr_note)
    }

    pub fn eval_template_with_note(
        &mut self,
        code: impl Into<AutoStr>,
        note: char,
    ) -> AutoResult<Value> {
        self.evaler.set_mode(EvalMode::TEMPLATE);
        let code = code.into();
        let flipped = flip_template(code.as_str(), note);
        let mut parser = Parser::new_with_note(flipped.as_str(), self.scope.clone(), note);
        let ast = parser.parse()?;
        let result = self.evaler.eval(&ast)?;
        Ok(result)
    }

    pub fn load_file(&mut self, filename: &str) -> AutoResult<Value> {
        let code =
            std::fs::read_to_string(filename).map_err(|e| format!("Failed to read file: {}", e))?;
        self.interpret(&code)?;
        Ok(self.result.clone())
    }

    pub fn load_config(&mut self, filename: &str) -> AutoResult<Value> {
        let code =
            std::fs::read_to_string(filename).map_err(|e| format!("Failed to read file: {}", e))?;
        self.result = self.eval_config(&code)?;
        self.scope
            .borrow_mut()
            .set_global("result", self.result.clone());
        Ok(self.result.clone())
    }

    fn eval_config(&mut self, code: &str) -> AutoResult<Value> {
        let mut parser = Parser::new_with_note(code, self.scope.clone(), self.fstr_note);
        let ast = parser.parse().map_err(|e| e.to_string())?;
        let mut config_evaler = Evaler::new(self.scope.clone()).with_mode(EvalMode::CONFIG);
        config_evaler.register_with_universe();
        let res = config_evaler.eval(&ast)?;
        Ok(res)
    }

    pub fn eval(&mut self, code: &str) -> Value {
        let mut parser = Parser::new_with_note(code, self.scope.clone(), self.fstr_note);
        let ast = parser.parse();
        match ast {
            Ok(ast) => {
                let mut val = Value::Nil;
                for stmt in ast.stmts {
                    val = match self.evaler.eval_stmt(&stmt) {
                        Ok(v) => v,
                        Err(e) => Value::Error(format!("Evaluation error: {:?}", e).into()),
                    };
                }
                if let Value::ValueRef(id) = val {
                    // lookup ValueData as Value
                    let data = self.scope.borrow().clone_value(id).unwrap();
                    Value::from_data(data)
                } else {
                    val
                }
            }
            Err(err) => {
                let msg = format!("AutoError: {}", err);
                Value::Error(msg.into())
            }
        }
    }
}

// convert template (ex, a C file with interpolated auto expressions) into an auto source code with C code converted to lines of interpolated strings
// Example:
// template:
// <code>
// #include <stdio.h>
// int main() {
//     printf("Hello, $name!\n");
//     return 0;
// }
// </code>
// flipped:
// <code>
// f`#include <stdio.h>`
// f`int main() {`
// f`    printf(\"Hello, $name!\\n\");`
// f`    return 0;`
// f`}`
// </code>
pub fn flip_template(template: &str, fnote: char) -> String {
    let lines = template.lines();
    let mut result = Vec::new();
    for line in lines {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            // NOTE: keep empty lines
            result.push("``".to_string());
            continue;
        }
        let fnote_space = format!("{} ", fnote);
        let fnote_brace_pat = format!("{}{{", fnote);
        // Only treat lines starting with "$ " (dollar + space) as code
        // Lines starting with "$word" are content with embedded variable
        if trimmed.starts_with(&fnote_space) || trimmed.starts_with(fnote_brace_pat.as_str()) {
            // Code line: remove the leading "$ " or wrap the whole line
            if trimmed.starts_with(&fnote_space) {
                let code = &trimmed[2..]; // Skip "$ "
                result.push(format!("{}", code));
            } else {
                // Starts with "${" - keep as-is for f-string processing
                result.push(format!("`{}`", line));
            }
        } else {
            result.push(format!("`{}`", line));
        }
    }
    // str.lines() does not include the last empty line
    if template.ends_with("\n") {
        result.push("``".to_string());
    }
    result.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flip_template_simple() {
        let code = r#"
$ for i in 0..10 {
    ${i}${mid{","}}
$ }"#;
        let result = flip_template(code, '$');

        assert_eq!(result, "``\nfor i in 0..10 {\n`    ${i}${mid{\",\"}}`\n}",);
    }

    #[test]
    fn test_flip_template() {
        let code = r#"#include <stdio.h>

int main() {
    printf("Hello, $name!\n");

    $ for i in 0..10 {
        printf("i = $i\n");
    $ }

    return 0;
}
"#;
        let result = flip_template(code, '$');

        assert_eq!(
            result,
            r#"`#include <stdio.h>`
``
`int main() {`
`    printf("Hello, $name!\n");`
``
for i in 0..10 {
`        printf("i = $i\n");`
}
``
`    return 0;`
`}`
``"#
        );
    }

    #[test]
    fn test_flip_template_with_multiple_lines() {
        let template = r#"
$ for row in rows {
{
    name: ${row.name},
    age: ${row.age},
}${mid(",")}
$ }
"#;
        let result = flip_template(template, '$');
        assert_eq!(
            result,
            r#"``
for row in rows {
`{`
`    name: ${row.name},`
`    age: ${row.age},`
`}${mid(",")}`
}
``"#
        );
    }

    #[test]
    fn test_eval_template_with_note() {
        let code = r#"<workspace>
    <project>
        <path>$WS_DIR$\#{name}.ewp</path>
    </project>
    <batchBuild />
</workspace>
        "#;
        let mut scope = Universe::new();
        scope.set_global("name", "demo_project".into());
        let mut inter = Interpreter::with_scope(scope).with_fstr_note('#');
        let result = inter.eval_template_with_note(code, '#').unwrap();
        assert_eq!(
            result.repr(),
            r#"<workspace>
    <project>
        <path>$WS_DIR$\demo_project.ewp</path>
    </project>
    <batchBuild />
</workspace>
"#
        );
    }

    #[test]
    fn test_flip_template_with_note() {
        let code = r#"
@ for lib in libs {
    <name>@{lib.name}</name>
@ }
"#;
        let result = flip_template(code, '@');
        assert_eq!(
            result,
            r#"``
for lib in libs {
`    <name>@{lib.name}</name>`
}
``"#
        );
    }

    #[test]
    fn test_starts_with() {
        let s = "@ f";
        let c = '@';
        let pat = format!("{}", c);
        assert!(s.starts_with(pat.as_str()));

        let s1 = "@ for i in 0..10 {";
        assert!(s1.starts_with(pat.as_str()));
    }
}
