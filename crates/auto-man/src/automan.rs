use crate::asset::Templates;
use crate::AutoResult;
use crate::Pac;
use crate::Port;
use crate::TargetKind;
use crate::{Index, IndexStore};
use auto_lang::config::AutoConfig;
use auto_lang::Universe;
use auto_val::shared;
use auto_val::{AutoPath, AutoStr, Obj, Value};
use log::*;
use reqwest::blocking::get;
use std::collections::HashMap;
use std::env;
use std::fmt::Display;
use std::path::Path;

pub struct Automan {
    pac: Pac,
    index_store: IndexStore,
}

// Static API
impl Automan {
    pub fn create_app(name: &str) -> AutoResult<()> {
        Self::create_by_template(name, "app")
    }

    pub fn create_capp(name: &str) -> AutoResult<()> {
        Self::create_by_template(name, "capp")
    }

    pub fn create_lib(name: &str) -> AutoResult<()> {
        Self::create_by_template(name, "lib")
    }

    pub fn create_clib(name: &str) -> AutoResult<()> {
        Self::create_by_template(name, "clib")
    }

    pub fn create_by_template(name: &str, template: &str) -> AutoResult<()> {
        // 1. Check if project already exists
        let path = Path::new(name);
        if path.is_file() {
            return Err(format!("A file named with {} already exists", name).into());
        }
        if path.is_dir() {
            // TODO: Let the user choose to replace the directory
            return Err(format!("A directory named with {} already exists", name).into());
        }
        // 2. Copy template to destination
        Templates::copy(template, name)?;
        Ok(())
    }

    pub fn reset_index() -> AutoResult<()> {
        // rewrite am.at
        let home = dirs::home_dir().ok_or("Can't open home dir")?;
        let auto_dir = home.join(".auto").join("auto-man");
        let am_at_file = auto_dir.join("am.at");
        let am_url =
            "https://gitee.com/auto-stack/auto-man/raw/master/crates/auto-man/assets/am.at";
        let page = get(am_url)?.text()?;
        std::fs::write(&am_at_file, page)?;
        info!("updated {}", am_at_file.display());
        // remove index dirs
        let index_dir = auto_dir.join("index");
        if index_dir.exists() {
            std::fs::remove_dir_all(&index_dir)?;
            info!("removed index dir {}", index_dir.display());
        }
        Ok(())
    }

    pub fn install_config(file: &str) -> AutoResult<()> {
        let home = dirs::home_dir().ok_or("Can't open home dir")?;
        let auto_dir = home.join(".auto").join("auto-man");
        let am_at_file = auto_dir.join("am.at");
        let path = Path::new(file);
        if !path.exists() {
            return Err(format!("am.at config file {} not found", file).into());
        }
        let content = std::fs::read_to_string(path)?;
        std::fs::write(&am_at_file, content)?;
        info!("updated {}", am_at_file.display());
        Ok(())
    }
}

// API
impl Automan {
    pub fn parse_pac(path: &str, am_config: &AmConfig) -> AutoResult<Self> {
        // 1. Find pac.at
        let config_path = Path::new(path).join("pac.at");
        if !config_path.is_file() {
            return Err(format!("No pac.at file found in {}", path).into());
        }

        // 2. try load saved port
        let port = Self::try_load_port()?;

        // let user select a port
        let config = AutoConfig::read(config_path.as_path())?;
        let ports = config.root.get_nodes("port");

        let mut port_names = Vec::new();
        for port in ports {
            port_names.push(port.main_arg().to_astr());
        }

        let dft_port: Port = Port::default();
        let port_name = match port {
            None => {
                if !port_names.contains(&dft_port.name) {
                    // default port is win32
                    port_names.push(dft_port.name);
                }
                Self::select_port(&port_names)?
            }
            Some(port) => {
                if !port_names.contains(&port) {
                    if !port_names.contains(&dft_port.name) {
                        // default port is win32
                        port_names.push(dft_port.name);
                    }
                    Self::select_port(&port_names)?
                } else {
                    port
                }
            }
        };

        println!("port NAME: {}", port_name);

        // use this port to reload config and make a pac
        let mut env = Universe::new();
        env.set_global("port", port_name.clone().into());

        let config = AutoConfig::from_file(config_path.as_path(), &Obj::new(), env)?;
        let mut index_used: Vec<AutoStr> = Vec::new();
        if config.root.has_prop("index") {
            index_used = config
                .root
                .get_prop("index")
                .as_array()
                .iter()
                .map(|t| t.to_astr())
                .collect();
        }
        if index_used.is_empty() {
            index_used.push("default".into());
        }
        let mut pac = Pac::new(config);
        let index_store = Self::index_store(am_config, index_used)?;
        info!("loading index store from {}", index_store.path.to_astr());
        pac.index = shared(Index::load(index_store.path.join("index.at").to_astr())?);
        pac.device_index = shared(Index::load(index_store.path.join("devices.at").to_astr())?);

        pac.set_port(port_name.clone())?;
        pac.save_port(port_name.clone())?;

        Ok(Self { pac, index_store })
    }

    fn select_port(ports: &Vec<AutoStr>) -> AutoResult<AutoStr> {
        crate::util::select_or_default_port(None, ports, "Which port do you want to build?")
    }

    pub fn get_state_file() -> AutoResult<AutoPath> {
        // check state file exists? .am/state.at
        let am_dir = AutoPath::new(".am");

        if !am_dir.is_dir() {
            // Create the directory if it doesn't exist
            std::fs::create_dir_all(&am_dir.path())?;
        }

        let state_file = am_dir.join("state.at");
        if !state_file.is_file() {
            // Create the state file if it doesn't exist
            std::fs::File::create(&state_file.path())?;
        }

        Ok(state_file)
    }

    pub fn try_load_port() -> AutoResult<Option<AutoStr>> {
        // load local state from .am/state.at
        let state_file = Self::get_state_file()?;
        let state = AutoConfig::read(&state_file.path())?;

        let port_name = state.root.get_prop("port").to_astr();
        println!("Loaded port: {}", port_name);
        if !port_name.is_empty() {
            Ok(Some(port_name))
        } else {
            Ok(None)
        }
    }

    pub fn new(path: &str, am: AmConfig) -> AutoResult<Self> {
        // Currently, only CMake builder is supported
        let am = Self::parse_pac(path, &am)?;
        Ok(am)
    }

    pub fn pull(&mut self) -> AutoResult<()> {
        self.index_store.pull()?;
        self.pac.pull()
    }

    pub fn set_port(&mut self, port: AutoStr) -> AutoResult<()> {
        self.pac.set_port(port.clone())?;
        self.pac.save_port(port)
    }

    pub fn open_ide(&mut self) -> AutoResult<()> {
        println!("build dir: {}", self.pac.build_location);
        println!("port: {}", self.pac.port.name);
        match self.pac.port.builder.as_str() {
            "iar" => {
                // open iar ide in the build_location
                let eww_path = AutoPath::new(self.pac.build_location.as_str())
                    .join(self.pac.name.to_string() + ".eww")
                    .to_astr()
                    .replace("/", "\\");
                println!("eww: {}", eww_path.clone());
                std::process::Command::new("explorer.exe")
                    .arg(eww_path.as_str())
                    .output()?;
            }
            "ghs" => {
                let ghs_path = AutoPath::new(self.pac.build_location.as_str())
                    .join("default.gpj")
                    .to_astr()
                    .replace("/", "\\");
                println!("ghs: {}", ghs_path.clone());
                std::process::Command::new("explorer.exe")
                    .arg(ghs_path.as_str())
                    .output()?;
            }
            "cmake" => {
                // in windows, there should be a .sln project in build dir
                let sln_path = AutoPath::new(self.pac.build_location.as_str())
                    .join("build")
                    .join(self.pac.name.to_string() + ".sln")
                    .to_astr()
                    .replace("/", "\\");
                println!("sln: {}", sln_path.clone());
                std::process::Command::new("explorer.exe")
                    .arg(sln_path.as_str())
                    .output()?;
            }
            _ => {
                println!("port builder: {}", self.pac.port.builder);
                todo!()
            }
        }
        Ok(())
    }

    pub fn scan(&mut self) -> AutoResult<()> {
        // self.pac.print_targets();
        // 3. Resolve targets in the pac, including:
        // 3.1. Resolve dependencies
        // 3.2. Link targets with dependencies
        // 3.3. Scan targets for its local source files
        self.pac.resolve()?;
        self.pac.print_targets();
        self.pac.print_atom()
    }

    pub fn info(&self, _target: Option<String>) -> AutoResult<()> {
        // read content from pac.atom.at
        let config = AutoConfig::read(Path::new("pac.atom.at"))?;
        let pac = Pac::new(config);
        pac.print_targets();
        Ok(())
        // if let Some(target) = target {
        //     self.pac.print_target_info(&target)
        // } else {
        //     self.pac.print_targets();
        //     self.pac.print_atom()
        // }
    }

    pub fn build(&mut self) -> AutoResult<()> {
        // 1. Transpile auto code to c code
        println!("Transpiling auto code to c code");
        self.transpile_auto()?;
        // 2. Build the project with a specific builder
        self.pac.build()
    }

    pub fn transpile_auto(&mut self) -> AutoResult<()> {
        self.pac.transpile_autot()?;
        Ok(())
    }

    pub fn run(&mut self, args: Vec<String>) -> AutoResult<()> {
        self.pac.run(args)
    }

    pub fn clean(&mut self) -> AutoResult<()> {
        // logs
        let files = glob::glob("*.log")?;
        for file in files {
            if let Ok(file) = file {
                info!("deleting file {}", file.display());
                std::fs::remove_file(file)?;
            }
        }

        // clean downloaded deps with pac.atom.at
        let atom_file = AutoPath::new(".am/pac.atom.at");
        if !atom_file.is_file() {
            return Err(format!(
                "{}\n{}\n{}",
                "Build info file .am/pac.atom.at not found!",
                "`am clean` depends on this info file to complete.",
                "Please run `am b` first to generate build info file."
            )
            .into());
        }
        let history_config = AutoConfig::read(atom_file.path())?;
        let mut history_pac = Pac::new(history_config);
        for t in history_pac
            .targets
            .iter_mut()
            .filter(|t| t.kind == TargetKind::Dep || t.kind == TargetKind::Device)
        {
            println!("target [{}] at {} ...", t.name, t.at);
            match t.clean() {
                Ok(_) => info!("Target [{}] at {} cleaned successfully", t.name, t.at),
                Err(e) => {
                    error!("Error: {}", e);
                    continue;
                }
            }
        }

        // clean .am folder
        std::fs::remove_dir_all(".am")?;
        // clean pac related files
        self.pac.clean()
    }

    pub fn list_port_names(&self) -> Vec<AutoStr> {
        self.pac.list_port_names()
    }

    pub fn list_deps(config: &AmConfig) -> AutoResult<()> {
        let store = Self::index_store(config, vec![])?;
        store.list_deps()
    }

    pub fn index_store(config: &AmConfig, indexs: Vec<AutoStr>) -> AutoResult<IndexStore> {
        // try to get index location from amconfig
        let am_path = home_path().join(".auto/auto-man");
        if !am_path.is_dir() {
            info!("creating am dir: {}", am_path.to_astr());
            std::fs::create_dir_all(am_path.path())?;
        }
        // get index name and urls from config and indexs
        let all_index = &config.index;
        println!("all index: {:?}", all_index);
        let mut used_indexs = indexs;
        if used_indexs.is_empty() {
            // fill all index
            for k in all_index.keys() {
                used_indexs.push(k.clone());
            }
        }
        println!("used index: {:?}", used_indexs);

        let mut used_index_path = AutoPath::new(".");

        for index in used_indexs {
            let index_path = am_path.join("index").join(index.clone());
            used_index_path = index_path.clone();
            let repo = all_index.get(&index);
            let Some(repo) = repo else {
                error!(
                    "index base not found! {}, please check ~/.auto/auto-man/am.at",
                    index
                );
                continue;
            };
            if !index_path.is_dir() {
                // try to clone
                info!("cloning index dir: {}", index_path.to_astr());
                std::fs::create_dir_all(index_path.path())?;

                let result = std::process::Command::new("git")
                    .arg("clone")
                    .arg(repo.as_str())
                    .arg(index_path.path())
                    .output();
                match result {
                    Err(e) => {
                        error!(
                            "Failed to clone repository {} to {} with error {}",
                            repo, index_path, e
                        );
                    }
                    Ok(_) => {}
                }
            }
        }
        // else {
        // // try to update
        // info!("updating index dir: {}", index_path.to_astr());
        // let result = std::process::Command::new("git")
        //     .arg("pull")
        //     .arg("--rebase")
        //     .current_dir(index_path.path())
        //     .output();
        // match result {
        //     Err(e) => {
        //         let repo = config.index.as_str();
        //         error!(
        //             "Failed to update repository {} to {} with error {}",
        //             repo, index_path, e
        //         );
        //     }
        //     Ok(_) => {}
        // }
        // }
        return Ok(IndexStore::new(used_index_path));

        // let dir = Self::search_dirs_or_clone(
        //     config,
        //     ".am/index",
        //     vec!["D://".into(), home_path(), "C://".into(), "E://".into()],
        // );
        // println!("dir: {:?}", dir);
        // if let Some(dir) = dir {
        //     Ok(IndexStore::new(dir))
        // } else {
        //     Err(format!("No index found").into())
        // }
    }

    pub fn list_devices(config: &AmConfig) -> AutoResult<()> {
        let store = Self::index_store(config, vec![])?;
        store.list_devices()
    }
}

fn home_path() -> AutoPath {
    dirs::home_dir().unwrap().to_str().unwrap().into()
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_search_dirs() {
        // let path = search_dirs_or_clone(
        //     ".am/index",
        //     vec!["D://".into(), home_path(), "C://".into(), "E://".into()],
        // );
        // assert!(path.is_some());
        // assert!(path.unwrap().is_dir());
    }

    #[test]
    fn test_env() {
        use std::env;
        println!("home: {}", env::var("HOME").unwrap());
    }
}

pub struct AmConfig {
    pub name: AutoStr,
    pub index: HashMap<AutoStr, AutoStr>,
    pub is_default: bool,
}

impl Default for AmConfig {
    fn default() -> Self {
        let mut map = HashMap::new();
        map.insert(
            "default".into(),
            "git@gitee.com:auto-stack/auto-index.git".into(),
        );
        Self {
            name: "default".into(),
            index: map,
            is_default: true,
        }
    }
}

impl Display for AmConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "AmConfig {{")?;
        writeln!(f, "  name: {}", self.name)?;
        writeln!(f, "  index: {{")?;
        for (k, v) in self.index.iter() {
            writeln!(f, "    {}: {}", k, v)?;
        }
        writeln!(f, "  }}")?;
        writeln!(f, "}}\n")
    }
}

pub fn load_am_config() -> Option<AmConfig> {
    // try to look for am.at file in the working directory
    let mut am_path = None;

    let local_config_path = env::current_dir().unwrap().join("am.at");
    if local_config_path.is_file() {
        am_path = Some(local_config_path);
    }

    if am_path.is_none() {
        let user_home_config_path = dirs::home_dir()
            .unwrap()
            .join(".auto")
            .join("auto-man")
            .join("am.at");
        if !user_home_config_path.is_dir() {
            // create parent dir
            let rs = std::fs::create_dir_all(user_home_config_path.parent().unwrap());
            if rs.is_err() {
                error!("Failed to create parent directory for am.at file");
            }
        }
        if user_home_config_path.is_file() {
            am_path = Some(user_home_config_path);
        } else {
            info!("Automan Config file am.at not found in current or home directory, using the default one");
            // write default config to am.at file
            let default_config = AmConfig::default();
            let default_config_str = default_config.to_string();
            std::fs::write(user_home_config_path.clone(), default_config_str).unwrap();
            am_path = Some(user_home_config_path);
        }
    }

    let Some(am_path) = am_path else {
        return None;
    };
    let code = std::fs::read_to_string(am_path.clone());
    let Ok(code) = code else {
        return None;
    };
    let config = AutoConfig::new(code);
    let Ok(config) = config else {
        return None;
    };
    let mut am = AmConfig::default();
    if config.root.has_prop("index") {
        let index_prop = config.root.get_prop("index");
        if let Value::Obj(obj) = index_prop {
            am.index = obj.to_hashmap();
        }
    }
    am.is_default = false;
    println!("Loading AmConfig from {:?}...", am_path.to_str());
    println!("AmConfig: {}", am);
    Some(am)
}
