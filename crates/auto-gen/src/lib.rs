// Modules
mod data;
mod error;
mod generator;
mod guard;
mod template;

// Re-exports
pub use data::{DataLoader, DataSource};
pub use error::{GenError, GenResult, SourceLocation};
pub use generator::{
    CodeGenerator, CodeGeneratorBuilder, GenReport, GenerationSpec, GeneratorConfig, TemplateSpec,
};
pub use guard::{Conflict, GuardProcessor, GuardedSection};
pub use template::{Template, TemplateEngine};

// ============================================================================
// LEGACY CODE - Kept for backward compatibility during transition
// ============================================================================

use std::{collections::HashMap, path::Path};

use auto_atom::Atom;
use auto_val::{AutoPath, AutoResult, AutoStr};
use regex::Regex;

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

impl Default for AutoGen {
    fn default() -> Self {
        Self::new()
    }
}

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
        let atom_name = self.data.name.clone();
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
        let mut universe = auto_lang::Universe::new();
        universe.merge_atom(&self.data);
        let mut inter =
            auto_lang::interp::Interpreter::with_scope(universe).with_fstr_note(self.note);
        let result = inter.eval_template(&mold.code);
        match result {
            Ok(result) => result.to_astr(),
            Err(e) => {
                println!("error: {}", e);
                mold.code.clone()
            }
        }
    }

    fn feed_guarded(file: &Path, input: AutoStr) -> AutoStr {
        if !file.is_file() {
            return input;
        }
        let mut current = String::new();
        let mut guarded_contents: HashMap<AutoStr, AutoStr> = HashMap::new();
        let mut key: AutoStr = AutoStr::new();
        let mut is_in_guard = false;

        let starter = "/// ---------- begin of guard:";
        let ender = "/// ---------- end of guard:";
        // read contents of the file line by line
        for line in std::fs::read_to_string(file).unwrap().lines() {
            if line.starts_with(starter) {
                // get key
                key = capture_block(line);
                is_in_guard = true;
            } else if line.starts_with(ender) {
                // ending
                if !key.is_empty() {
                    guarded_contents.insert(key.clone(), current.clone().into());
                }
                current.clear();
                is_in_guard = false;
            } else {
                if is_in_guard {
                    current.push_str(line);
                    current.push('\n');
                }
            }
        }

        // find guarded contents, starting with /// AG_GUARD <GUARD_ID>
        let mut outputs: Vec<AutoStr> = Vec::new();
        let mut is_out_guard = false;
        for line in input.lines() {
            if line.starts_with(starter) {
                is_out_guard = true;
                outputs.push(line.into());
                // get key
                // pattern /// AG_GUARD <GUARD_ID> {{{
                let key = capture_block(line);
                if guarded_contents.contains_key(&key) {
                    guarded_contents.remove(&key).map(|c| {
                        for l in c.lines() {
                            outputs.push(l.into());
                        }
                    });
                }
            } else if line.starts_with(ender) {
                is_out_guard = false;
                outputs.push(line.into());
            } else {
                if !is_out_guard {
                    outputs.push(line.into());
                }
            }
        }
        outputs.join("\n").into()
    }

    fn gen_one(&self, mold: &Mold, out_file: &AutoPath) {
        let mut universe = auto_lang::Universe::new();
        universe.merge_atom(&self.data);
        let mut inter =
            auto_lang::interp::Interpreter::with_scope(universe).with_fstr_note(self.note);
        let result = inter.eval_template(&mold.code);
        match result {
            Ok(result) => {
                let out_str = result.to_astr();
                let feeded = Self::feed_guarded(&out_file.path(), out_str);
                std::fs::write(out_file.path(), feeded.as_bytes()).unwrap();
                println!("generated: {}", out_file.to_astr());
            }
            Err(e) => {
                println!("error: {}", e);
                let code = if mold.code.len() > 100 {
                    (mold.code[..100].to_string() + "...").into()
                } else {
                    mold.code.clone()
                };
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
        let mut universe = auto_lang::Universe::new();
        universe.merge_atom(&self.data);
        let mut inter =
            auto_lang::interp::Interpreter::with_scope(universe).with_fstr_note(self.note);
        let result = inter.eval_template(&self.mold.code);
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

fn capture_block(line: &str) -> AutoStr {
    let mut key = "".into();
    let pat = Regex::new(r"guard: <(\w+)> ---").unwrap();
    if let Some(caps) = pat.captures(line) {
        key = caps[1].to_string().into();
    }
    key
}

#[cfg(test)]
mod tests {
    use auto_val::Value;

    use super::*;

    #[test]
    fn test_gen() {
        let values = vec![Value::pair("a", 1), Value::pair("b", 2)];
        let atom = Atom::assemble(values);
        let ag = AutoGen::new().data(atom);
        let result = ag.gen_all();
        assert_eq!(result, "a: 1; b: 2");
    }

    #[test]
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
    fn test_capture_guard() {
        let line = "/// ---------- begin of guard: <includes> -----------------------------------";
        let block = capture_block(line);
        assert_eq!(block, "includes");
    }
}
