// Modules
mod data;
mod generator;
mod template;

#[cfg(test)]
mod test_framework;

// Re-exports — the guard/error/report types and TemplateSpec have a single
// definition in auto-gen-core (PLAN-791); the historical public paths of this
// crate are preserved via re-export.
pub use auto_gen_core::{
    Conflict, GenError, GenReport, GenResult, GuardProcessor, GuardedSection, SourceLocation,
    TemplateSpec,
};
pub use data::{DataLoader, DataSource, LoadedData};
pub use generator::{CodeGenerator, CodeGeneratorBuilder, GenerationSpec, GeneratorConfig};
pub use template::{Template, TemplateEngine};

// ============================================================================
// LEGACY CODE - Kept for backward compatibility during transition
// ============================================================================

use auto_val::{AutoPath, AutoResult, AutoStr};
use auto_lang::atom::Atom;

pub struct Mold {
    pub name: AutoStr,
    pub code: AutoStr,
    pub is_rename: bool,
}

impl Mold {
    pub fn new(name: impl Into<AutoStr>, code: impl Into<AutoStr>) -> Self {
        Self {
            name: name.into(),
            code: code.into(),
            is_rename: false,
        }
    }

    pub fn is_rename(mut self, is_rename: bool) -> Self {
        self.is_rename = is_rename;
        self
    }

    pub fn from_file(path: impl Into<AutoPath>) -> Self {
        let path = path.into();
        let code = std::fs::read_to_string(path.path()).unwrap();
        let name = path.filename();
        Self::new(name, code)
    }
}

#[deprecated(note = "Use CodeGenerator instead")]
pub struct AutoGen {
    pub data: Atom,
    pub out: AutoPath,
    pub molds: Vec<Mold>,
    pub is_rename: bool,
    note: char,
}

#[allow(deprecated)]
impl Default for AutoGen {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(deprecated)]
impl AutoGen {
    pub fn new() -> Self {
        Self {
            data: Atom::default(),
            out: AutoPath::new("."),
            molds: Vec::new(),
            note: '$',
            is_rename: false,
        }
    }

    pub fn data(mut self, data: Atom) -> Self {
        self.data = data;
        self
    }

    pub fn note(mut self, note: char) -> Self {
        self.note = note;
        self
    }

    pub fn rename(mut self, is_rename: bool) -> Self {
        self.is_rename = is_rename;
        self
    }

    pub fn out(mut self, path: impl Into<AutoPath>) -> Self {
        self.out = path.into();
        self
    }

    pub fn molds(mut self, molds: Vec<Mold>) -> Self {
        self.molds = molds;
        self
    }

    pub fn mold(mut self, mold: Mold) -> Self {
        self.molds.push(mold);
        self
    }

    // Main API
    pub fn gen_all(&self) -> AutoStr {
        // Extract name from atom - if it's a Node, get its name property
        let atom_name = match &self.data {
            auto_lang::atom::Atom::Node(node) => {
                let name_prop = node.get_prop_of("name");
                if !name_prop.is_nil() {
                    name_prop.to_astr().to_string()
                } else {
                    node.main_arg().to_astr().to_string()
                }
            }
            _ => String::new(),
        };

        for mold in self.molds.iter() {
            let out_file = if self.is_rename || mold.is_rename {
                let out_name = replace_name(mold.name.clone(), atom_name.clone());
                self.out.join(&out_name)
            } else {
                let trimmed = mold.name.replace(".at.", ".");
                self.out.join(&trimmed)
            };
            self.gen_one(&mold, &out_file);
        }
        self.data.to_astr()
    }

    pub fn gen_str(&self) -> AutoStr {
        let mut result = String::new();
        for mold in self.molds.iter() {
            let code = self.gen_one_str(&mold);
            result.push_str(&code);
        }
        result.into()
    }

    fn gen_one_str(&self, mold: &Mold) -> AutoStr {
        let mut interp = auto_lang::interpreter::AutoInterpreter::new().with_fstr_note(self.note);
        interp.merge_atom(&self.data);
        let result = interp.eval_template("", &mold.code);
        match result {
            Ok(result) => result.to_astr(),
            Err(e) => {
                println!("error: {}", e);
                mold.code.clone()
            }
        }
    }

    fn gen_one(&self, mold: &Mold, out_file: &AutoPath) {
        let mut interp = auto_lang::interpreter::AutoInterpreter::new().with_fstr_note(self.note);
        interp.merge_atom(&self.data);
        let result = interp.eval_template("", &mold.code);
        match result {
            Ok(result) => {
                let out_str = result.to_astr();
                // Legacy guard feed lives in auto-gen-core as the single
                // byte-exact definition (PLAN-791): join without trailing
                // newline, exact marker prefixes, absent file passes input
                // through.
                let feeded = auto_gen_core::feed_guarded_legacy(&out_file.path(), out_str);
                std::fs::write(out_file.path(), feeded.as_bytes()).unwrap();
                println!("generated: {}", out_file.to_astr());
            }
            Err(e) => {
                println!("error: {:?}", e);
                // pretty print the error
                let code = if mold.code.len() > 100 {
                    (mold.code[..100].to_string() + "...").into()
                } else {
                    mold.code.clone()
                };
                println!("code: {}", code);
                panic!("failed to evaluate mold: {}", code);
            }
        }
    }
}

#[deprecated(note = "Use CodeGenerator instead")]
pub struct OneGen {
    pub data: Atom,
    pub out: AutoPath,
    pub mold: Mold,
    pub is_rename: bool,
    note: char,
}

#[allow(deprecated)]
impl OneGen {
    pub fn new(mold: Mold, data: Atom) -> Self {
        Self {
            out: AutoPath::new("."),
            mold,
            note: '$',
            is_rename: false,
            data,
        }
    }
}

#[allow(deprecated)]
impl OneGen {
    pub fn note(mut self, note: char) -> Self {
        self.note = note;
        self
    }

    pub fn is_rename(mut self, is_rename: bool) -> Self {
        self.is_rename = is_rename;
        self
    }

    pub fn data(mut self, data: Atom) -> Self {
        self.data = data;
        self
    }

    pub fn mold(mut self, mold: Mold) -> Self {
        self.mold = mold;
        self
    }

    pub fn out(mut self, out: impl Into<AutoPath>) -> Self {
        self.out = out.into();
        self
    }

    pub fn gen(&self) -> AutoResult<()> {
        let mut interp = auto_lang::interpreter::AutoInterpreter::new().with_fstr_note(self.note);
        interp.merge_atom(&self.data);
        let result = interp.eval_template("", &self.mold.code);
        match result {
            Ok(result) => {
                let out_str = result.to_astr();
                let path = self.out.join(self.mold.name.clone());
                let parent_dir = path.parent();
                if !parent_dir.is_dir() {
                    std::fs::create_dir_all(parent_dir.path())?;
                }
                println!("writing to {}", path.to_astr());
                std::fs::write(path.path(), out_str.as_bytes()).unwrap();
                println!("generated: {}", self.out.to_astr());
                Ok(())
            }
            Err(e) => {
                println!("error: {}", e);
                let code = if self.mold.code.len() > 100 {
                    (self.mold.code[..100].to_string() + "...").into()
                } else {
                    self.mold.code.clone()
                };
                panic!("failed to evaluate mold: {}", code);
            }
        }
    }
}

// Static methods
#[allow(deprecated)]
impl AutoGen {
    pub fn is_template_file(path: &AutoPath) -> bool {
        let exts = path.exts(2);
        return exts.len() == 2 && exts[0] == "txt" && exts[1] == "at";
    }
}

fn replace_name(name: impl Into<AutoStr>, replace: impl Into<AutoStr>) -> AutoStr {
    let ext = AutoPath::new(name.into()).ext();
    format!("{}.{}", replace.into(), ext).into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use auto_val::Value;

    #[test]
    #[allow(deprecated)]
    fn test_gen() {
        let values = vec![Value::pair("a", 1), Value::pair("b", 2)];
        let atom = Atom::assemble(values).unwrap();
        let ag = AutoGen::new().data(atom);
        let result = ag.gen_all();
        assert_eq!(result, "atom {a: 1; b: 2}");
    }

    #[test]
    #[allow(deprecated)]
    fn test_is_template() {
        let path = "test.at.txt";
        let ap = AutoPath::new(path);
        assert_eq!(AutoGen::is_template_file(&ap), true);
    }

    #[test]
    fn test_replace_name() {
        let name = "iar.eww";
        let replace = "hello";
        let result = replace_name(name, replace);
        assert_eq!(result, "hello.eww");
    }

    #[test]
    fn test_autogen_markdown_tests() {
        use std::path::PathBuf;
        use test_framework::run_gen_tests_from_file;

        let test_file = PathBuf::from("tests/autogen_tests.md");
        match run_gen_tests_from_file(&test_file) {
            Ok(()) => println!("All auto-gen tests passed!"),
            Err(e) => panic!("Auto-gen tests failed:\n{}", e),
        }
    }
}
