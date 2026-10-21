use auto_val::astr_empty;
use auto_val::AutoPath;
use auto_val::AutoStr;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ToolLocation {
    Env,           // put in PATH environment variable
    Dir(AutoPath), // put in a specific directory
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ToolKind {
    None,
    Msvc,
    Gcc,
    Clang,
    IarArm,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Tool {
    pub kind: ToolKind,
    pub name: AutoStr,
    pub compiler: AutoStr,
    pub assembler: AutoStr,
    pub linker: AutoStr,
    pub archiver: AutoStr,
    pub location: ToolLocation,
}

impl Tool {
    pub fn set_path(&mut self, path: &str) {
        self.location = ToolLocation::Dir(AutoPath::from(path));
    }

    pub fn exe(&self, exe: AutoStr) -> AutoStr {
        match self.location {
            ToolLocation::Env => exe,
            ToolLocation::Dir(ref dir) => dir.join(exe).into(),
        }
    }

    pub fn cc(&self) -> AutoStr {
        match self.kind {
            ToolKind::None => astr_empty(),
            ToolKind::Msvc => format!("{} /c ", self.exe(self.compiler.clone())).into(),
            ToolKind::Gcc | ToolKind::Clang => {
                format!("{} -c ", self.exe(self.compiler.clone())).into()
            }
            ToolKind::IarArm => format!("{} -c ", self.exe(self.compiler.clone())).into(),
        }
    }

    pub fn defines(&self, defines: &Vec<AutoStr>) -> AutoStr {
        let define_cmd = match self.kind {
            ToolKind::Msvc => "/D",
            _ => "-D",
        };
        defines
            .iter()
            .map(|d| format!("{}{}", define_cmd, d)) // TODO: use toolset specific defines
            .collect::<Vec<String>>()
            .join(" ")
            .into()
    }
}

impl Tool {
    pub fn from(name: AutoStr, kind: AutoStr) -> Self {
        let kind = ToolKind::new(kind.as_str());
        return Tool::from_kind(name, kind);
    }

    pub fn from_kind(name: AutoStr, kind: ToolKind) -> Self {
        match kind {
            ToolKind::None => Tool {
                kind,
                name,
                compiler: astr_empty(),
                assembler: astr_empty(),
                linker: astr_empty(),
                archiver: astr_empty(),
                location: ToolLocation::Env,
            },
            ToolKind::Msvc => Tool {
                kind,
                name,
                compiler: AutoStr::from("cl"),
                assembler: AutoStr::from("ml64"),
                linker: AutoStr::from("link"),
                archiver: AutoStr::from("lib"),
                location: ToolLocation::Env,
            },
            ToolKind::Gcc => Tool {
                kind,
                name,
                compiler: AutoStr::from("gcc"),
                assembler: AutoStr::from("as"),
                linker: AutoStr::from("ld"),
                archiver: AutoStr::from("ar"),
                location: ToolLocation::Env,
            },
            ToolKind::Clang => Tool {
                kind,
                name,
                compiler: AutoStr::from("clang"),
                assembler: AutoStr::from("clang"),
                linker: AutoStr::from("ld"),
                archiver: AutoStr::from("ar"),
                location: ToolLocation::Env,
            },
            ToolKind::IarArm => Tool {
                kind,
                name,
                compiler: AutoStr::from("iccarm"),
                assembler: AutoStr::from("iasmarm"),
                linker: AutoStr::from("ilinkarm"),
                archiver: AutoStr::from("iarchive"),
                location: ToolLocation::Env,
            },
        }
    }

    pub fn none() -> Self {
        return Self::from_kind("none".into(), ToolKind::None);
    }
}

impl ToolKind {
    pub fn new(kind: &str) -> Self {
        match kind {
            "none" => ToolKind::None,
            "msvc" => ToolKind::Msvc,
            "gcc" => ToolKind::Gcc,
            "clang" => ToolKind::Clang,
            "iar-arm" => ToolKind::IarArm,
            _ => panic!("Unknown tool kind: {}", kind),
        }
    }
}
