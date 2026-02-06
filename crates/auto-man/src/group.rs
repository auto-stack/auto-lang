use auto_val::{AutoPath, AutoStr};
use std::collections::HashMap;
use std::fmt;

pub struct File {
    pub name: AutoStr,
}

pub struct Group {
    pub name: AutoStr,
    pub kids: HashMap<AutoStr, Group>,
    pub files: Vec<File>,
}

impl Group {
    pub fn new(name: &str) -> Self {
        Group {
            name: AutoStr::from(name),
            kids: HashMap::new(),
            files: Vec::new(),
        }
    }

    pub fn mut_kid(&mut self, name: &str) -> &mut Group {
        if self.kids.contains_key(name) {
            let kid = self.kids.get_mut(name).unwrap();
            kid
        } else {
            let kid = Group::new(name);
            self.kids.insert(name.into(), kid);
            self.kids.get_mut(name).unwrap()
        }
    }

    pub fn mut_kid_path(&mut self, path: &AutoPath) -> &mut Group {
        let mut g = self;
        for c in path.path().components() {
            let name = c.as_os_str().to_str().unwrap();
            let kid = g.mut_kid(name);
            g = kid;
        }
        g
    }

    pub fn print_kids(&self) {
        for (_n, kid) in &self.kids {
            println!("{}", kid);
        }
    }

    pub fn to_xml(&self) -> AutoStr {
        let mut xml = String::new();
        for (_n, kid) in &self.kids {
            xml.push_str(kid.to_string().as_str());
            xml.push_str("\n");
        }
        xml.into()
    }
}

impl fmt::Display for Group {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<group><name>{}</name>", self.name)?;
        // write kids
        for (_n, kid) in &self.kids {
            write!(f, "{}\n", kid)?;
        }
        // write files
        for file in &self.files {
            write!(f, "{}\n", file)?;
        }
        // write end tag
        write!(f, "</group>")?;
        Ok(())
    }
}

impl fmt::Display for File {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<file><name>{}</name></file>", self.name)?;
        Ok(())
    }
}

pub fn dir_groups(paths: Vec<AutoPath>) -> Group {
    let mut group = Group::new("root");
    for p in paths.iter() {
        group.mut_kid_path(p);
    }
    group
}
