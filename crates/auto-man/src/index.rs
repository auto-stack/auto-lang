use auto_lang::config::AutoConfig;
use auto_val::{AutoPath, AutoResult, AutoStr, Kid};
use std::collections::HashMap;
use std::fmt;
use std::path::Path;

use tabled::{
    settings::{object::Rows, style::Style, themes::Colorization, Color},
    Table, Tabled,
};

use crate::git::{check_changes, check_detached, pull};

#[derive(Clone, Debug)]
pub struct PacInfo {
    pub name: AutoStr,
    pub version: AutoStr,
    pub repo: AutoStr,
}

pub struct Index {
    map: HashMap<AutoStr, PacInfo>,
}

impl Default for Index {
    fn default() -> Self {
        Self {
            map: HashMap::default(),
        }
    }
}

impl fmt::Display for Index {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (name, info) in &self.map {
            writeln!(f, "{}: {}", name, info)?;
        }
        Ok(())
    }
}

impl fmt::Display for PacInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.name, self.version)
    }
}

impl Index {
    pub fn load(path: AutoStr) -> AutoResult<Self> {
        let path = Path::new(path.as_str());
        let mut map = HashMap::new();
        if !path.exists() {
            return Err(format!("index file {} not found", path.display()).into());
        }
        let code = std::fs::read_to_string(path)?;
        let config = AutoConfig::new(&code)?;
        // config is a bunch of index nodes
        let root = config.root;
        root.kids_iter()
            .filter_map(|(_, kid)| {
                if let Kid::Node(n) = kid {
                    if n.name == "pac" || n.name == "device" {
                        Some(n)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .for_each(|n| {
                let name = n.id();
                // let version = n.get_prop("version").to_astr();
                let versions = n.get_prop("versions");
                let version = versions.as_array().iter().last();
                let version = if let Some(v) = version {
                    v.to_astr()
                } else {
                    "".into()
                };
                let repo = n.get_prop("repo").to_astr();
                map.insert(
                    name.clone(),
                    PacInfo {
                        name,
                        repo,
                        version,
                    },
                );
            });

        Ok(Index { map })
    }

    pub fn lookup(&self, name: &AutoStr) -> Option<&PacInfo> {
        self.map.get(name)
    }
}

pub struct IndexStore {
    pub path: AutoPath,
}

#[derive(Tabled)]
pub struct PacIndex {
    name: AutoStr,
    url: AutoStr,
    versions: AutoStr,
}

impl IndexStore {
    pub fn new(path: AutoPath) -> Self {
        Self { path }
    }

    pub fn list_deps(&self) -> AutoResult<()> {
        // read index.at within the path's dir
        let index = self.path.join("index.at");
        if !index.is_file() {
            return Err(format!("No index.at file found in {}", self.path).into());
        }
        let index = AutoConfig::read(index.path())?;
        let mut pacs = Vec::new();
        for pac in index.root.nodes("pac") {
            pacs.push(PacIndex {
                name: pac.main_arg().to_astr(),
                url: pac.get_prop_of("repo").to_astr(),
                versions: pac.get_prop_of("versions").to_astr(),
            });
        }
        // print as table
        let mut table = Table::new(pacs);
        table
            .with(Style::modern_rounded())
            .with(Colorization::exact([Color::FG_BRIGHT_BLUE], Rows::first()));
        println!("{}", table);
        Ok(())
    }

    pub fn list_devices(&self) -> AutoResult<()> {
        let index = self.path.join("devices.at");
        if !index.is_file() {
            return Err(format!("No device.at file found in {}", self.path).into());
        }
        let index = AutoConfig::read(index.path())?;
        let mut devices = Vec::new();
        for device in index.root.nodes("device") {
            devices.push(PacIndex {
                name: device.main_arg().to_astr(),
                url: device.get_prop_of("repo").to_astr(),
                versions: device.get_prop_of("versions").to_astr(),
            });
        }
        // print as table
        let mut table = Table::new(devices);
        table
            .with(Style::modern_rounded())
            .with(Colorization::exact([Color::FG_BRIGHT_BLUE], Rows::first()));
        println!("{}", table);
        Ok(())
    }

    pub fn pull(&self) -> AutoResult<()> {
        check_changes(&self.path)?;
        check_detached(&self.path)?;
        pull(&self.path)?;
        Ok(())
    }
}
