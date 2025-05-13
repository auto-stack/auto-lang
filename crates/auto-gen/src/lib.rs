use auto_atom::Atom;
use auto_val::{AutoPath, AutoResult, AutoStr};

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

pub struct AutoGen {
    pub data: Atom,
    pub out: AutoPath,
    pub molds: Vec<Mold>, // paths to
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

    pub fn is_rename(mut self, is_rename: bool) -> Self {
        self.is_rename = is_rename;
        self
    }

    pub fn out(mut self, path: impl Into<AutoPath>) -> Self {
        let path = path.into();
        // println!("out path: {}", path.to_astr());
        self.out = path;
        // if path.is_dir() {
        // self.out = path;
        // } else {
        // panic!("output path {} must be a directory", path.to_astr());
        // }
        self
    }

    pub fn molds(mut self, molds: Vec<Mold>) -> Self {
        self.molds = molds;
        self
    }

    // Main API
    pub fn gen(&self) -> AutoStr {
        let atom_name = self.data.name.clone();
        for mold in self.molds.iter() {
            //TODO: rename mold to pac name
            let out_file = if self.is_rename || mold.is_rename {
                let out_name = replace_name(mold.name.clone(), atom_name.clone());
                self.out.join(&out_name)
            } else {
                self.out.join(&mold.name)
            };
            self.gen_one(&mold, &out_file);
        }
        self.data.to_astr()
    }

    pub fn gen_str(&self) -> AutoStr {
        let mut result = String::new();
        for mold in self.molds.iter() {
            //TODO: rename mold to pac name
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

    fn gen_one(&self, mold: &Mold, out_file: &AutoPath) {
        let mut universe = auto_lang::Universe::new();
        universe.merge_atom(&self.data);
        let mut inter =
            auto_lang::interp::Interpreter::with_scope(universe).with_fstr_note(self.note);
        let result = inter.eval_template(&mold.code);
        match result {
            Ok(result) => {
                let out_str = result.to_astr();
                std::fs::write(out_file.path(), out_str.as_bytes()).unwrap();
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

#[cfg(test)]
mod tests {
    use auto_val::Value;

    use super::*;

    #[test]
    fn test_gen() {
        let values = vec![Value::pair("a", 1), Value::pair("b", 2)];
        let atom = Atom::assemble(values);
        let ag = AutoGen::new().data(atom);
        let result = ag.gen();
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
}
