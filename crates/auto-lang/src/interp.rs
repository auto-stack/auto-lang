use crate::eval::{EvalMode, Evaler};
use crate::parser::Parser;
use crate::universe::Universe;
use crate::AutoResult;
use auto_val::{AutoStr, Value};
use std::cell::RefCell;
use std::rc::Rc;

pub struct Importer {
    pub path: AutoStr,
    pub scope: Rc<RefCell<Universe>>,
}

pub struct Interpreter {
    pub evaler: Evaler,
    pub scope: Rc<RefCell<Universe>>,
    pub result: Value,
    pub fstr_note: char,
}

impl Interpreter {
    pub fn new() -> Self {
        let scope = Rc::new(RefCell::new(Universe::new()));
        let interpreter = Self {
            evaler: Evaler::new(scope.clone()),
            scope,
            result: Value::Nil,
            fstr_note: '$',
        };
        interpreter
    }

    pub fn with_fstr_note(mut self, note: char) -> Self {
        self.fstr_note = note;
        self
    }

    pub fn with_scope(scope: Universe) -> Self {
        let scope = Rc::new(RefCell::new(scope));
        let interpreter = Self {
            evaler: Evaler::new(scope.clone()),
            scope,
            fstr_note: '$',
            result: Value::Nil,
        };
        interpreter
    }

    pub fn with_eval_mode(mut self, mode: EvalMode) -> Self {
        self.evaler = self.evaler.with_mode(mode);
        self
    }

    pub fn import(&mut self, path: AutoStr) -> Result<(), String> {
        println!("import: {}", path);
        Ok(())
    }

    pub fn interpret(&mut self, code: &str) -> Result<(), String> {
        let mut parser = Parser::new_with_note(code, self.scope.clone(), self.fstr_note);
        let ast = parser.parse()?;
        let result = self.evaler.eval(&ast);
        self.result = result;
        Ok(())
    }

    pub fn eval_template(&mut self, code: impl Into<AutoStr>) -> Result<Value, String> {
        self.eval_template_with_note(code, self.fstr_note)
    }

    pub fn eval_template_with_note(
        &mut self,
        code: impl Into<AutoStr>,
        note: char,
    ) -> Result<Value, String> {
        self.evaler.set_mode(EvalMode::TEMPLATE);
        let code = code.into();
        let flipped = flip_template(code.as_str(), note);
        let mut parser = Parser::new_with_note(flipped.as_str(), self.scope.clone(), note);
        let ast = parser.parse()?;
        let result = self.evaler.eval(&ast);
        Ok(result)
    }

    pub fn load_file(&mut self, filename: &str) -> Result<Value, String> {
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
        let res = config_evaler.eval(&ast);
        Ok(res)
    }

    pub fn eval(&mut self, code: &str) -> Value {
        match self.interpret(code) {
            Ok(_) => self.result.clone(),
            Err(e) => Value::error(e),
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
        let fnote_str = format!("{}", fnote);
        let fnote_brace_pat = format!("{}{{", fnote);
        if trimmed.starts_with(fnote_str.as_str()) && !trimmed.starts_with(fnote_brace_pat.as_str())
        {
            let code = &trimmed[1..].trim();
            result.push(format!("{}", code));
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
