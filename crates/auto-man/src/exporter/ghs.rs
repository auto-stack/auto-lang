#![allow(deprecated)]
use super::Exporter;
use crate::asset::GHSTemplate;
use crate::target::{Target, TargetKind};
use crate::Pac; // Changed from crate::pac::Pac (pac.rs not migrated yet)
use crate::{AutoResult, Dir};
use auto_gen::*;
use auto_lang::Atom;
use auto_val::{AutoPath, AutoStr};
use log::*;

pub struct GHSExporter {
    pub gen: AutoGen,
    pub apps: Vec<OneGen>,
    pub subprojects: Vec<OneGen>,
    pub path: AutoPath,
}

impl GHSExporter {
    pub fn new(path: AutoPath) -> Self {
        let gen = AutoGen::new().out(path.clone()).note('$');
        Self {
            gen,
            apps: Vec::new(),
            subprojects: Vec::new(),
            path,
        }
    }

    fn check_devices(&self, pac: &mut Pac) -> AutoResult<()> {
        let mut has_device = false;
        for target in &pac.targets {
            if target.kind == TargetKind::Device {
                has_device = true;
                break;
            }
        }
        if !has_device {
            // let mut dummy_device = Target::new("dummy", TargetKind::Device);
            // dummy_device.props.set("icf", "dummy");
            // dummy_device.props.set("ddf", "dummy");
            // dummy_device.props.set("board", "dummy");
            // pac.targets.push(dummy_device);
        }
        return Ok(());
    }
}

impl GHSExporter {
    fn setup_dir(&mut self, ghs_loc: AutoStr, dir: &Dir) -> AutoResult<()> {
        // let rel: AutoStr = format!("../{}", rel).into();
        // Implement directory setup logic here
        let mname = format!("{}.gpj", dir.lpath.clone());
        // replace / to _
        // let mname = mname.replace("/", "_");
        println!("MOLD name: {}", mname);
        let lib_mold = Mold::new(mname, GHSTemplate::subproject()?);
        let sub_dirs = &dir.dirs;
        let sub_names: Vec<AutoStr> = sub_dirs.iter().map(|dir| dir.lpath.clone()).collect();
        let mut dir_node = dir.to_node();
        dir_node.set_prop("dir_names", sub_names.clone());
        // 计算从.gpj到dir对应的实际目录之间的相对路径
        // rel_depth = [depth of ghs project] + [depth of .gpj]
        let rel_depth = self.path.depth() + dir.logical_depths() - 1;
        let rel = "../".repeat(rel_depth);
        dir_node.set_prop("rel", rel.clone());
        let atom = Atom::node(dir_node);
        let lib_gen = OneGen::new(lib_mold, atom).out(AutoPath::new(ghs_loc.clone()));
        self.subprojects.push(lib_gen);

        // recursively setup subdirectories
        for sub_dir in sub_dirs {
            self.setup_dir(ghs_loc.clone(), sub_dir)?;
        }
        Ok(())
    }
}

impl GHSExporter {
    fn setup(&mut self, pac: &mut Pac) -> AutoResult<()> {
        // check devices
        self.check_devices(pac)?;
        // check necessary configs
        // create iar project directory
        let project_dir = self.gen.out.path();
        if project_dir.is_file() {
            error!("{} is a file, not a directory", project_dir.display());
        } else if project_dir.is_dir() {
            // remove all files in the directory
            // std::fs::remove_dir_all(project_dir)?;
            std::fs::create_dir_all(project_dir)?;
        } else {
            std::fs::create_dir_all(project_dir)?;
        }
        // set data and molds for auto-gen
        let mut gen = std::mem::take(&mut self.gen);

        let mut rel = "".into();
        let mut device_dir = "".into();
        let mut device_inc = "".into();
        // find device target in pac
        let device_target = pac.targets.iter().find(|t| t.kind == TargetKind::Device);
        if let Some(device_target) = device_target {
            let device_loc = device_target.location();
            let ghs_loc = pac.build_location.clone();
            rel = AutoPath::new(ghs_loc).reverse_relative();
            device_dir = device_loc.to_astr();
            device_inc = device_loc.join("device").to_astr();
        } else {
            error!("Device target not found");
        }
        pac.props.set("rel", rel.clone());
        pac.props.set("device_dir", device_dir.clone());
        let atom = pac.to_atom();
        let molds = vec![Mold::new("default.gpj", GHSTemplate::default()?)];
        gen = gen.molds(molds);
        gen = gen.data(atom);
        self.gen = gen;

        // generators for apps
        let apps = pac.apps();
        let dep_names: Vec<AutoStr> = pac
            .targets
            .iter()
            .filter(|t| t.kind == TargetKind::Lib || t.kind == TargetKind::Dep)
            .map(|t| t.local_name().clone())
            .collect();

        // let device_names: Vec<AutoStr> = pac
        //     .targets
        //     .iter()
        //     .filter(|t| t.kind == TargetKind::Device)
        //     .map(|t| t.local_name().clone())
        //     .collect();

        // println!("DEVICES: {:?}", device_names);

        let mut all_incs = pac.all_incs();
        all_incs.sort();

        // let defines = pac.props.get_or_nil("defines");

        for app in apps {
            // let app_loc = app.location();
            let ghs_loc = pac.build_location.clone();
            println!("GHS Location: {}", ghs_loc);
            // let relative_loc = AutoPath::new(ghs_loc.clone()).reverse_relative();

            let mold_name = format!("{}.gpj", app.local_name());
            let app_mold = Mold::new(mold_name, GHSTemplate::program()?);
            let mut app_node = app.to_node();
            app_node.set_prop("local_name", app.local_name());
            // app_node.set_prop("libs", dep_names.clone());
            // app_node.set_prop("devices", device_names.clone());
            app_node.set_prop("rel", rel.clone());
            app_node.set_prop("device_dir", device_dir.clone());
            // app_node.set_prop("incs", all_incs.clone());
            app_node.set_prop("device_inc", device_inc.clone());
            // app_node.set_prop("defines", defines.clone());
            let atom = Atom::node(app_node);
            let app_gen = OneGen::new(app_mold, atom).out(AutoPath::new(ghs_loc.clone()));
            self.apps.push(app_gen);

            for (_key, dir) in &app.dirs {
                self.setup_dir(ghs_loc.clone(), dir)?;
            }

            for lib in &app.deps {
                let ghs_loc = pac.build_location.clone();
                let dir_names = lib
                    .dirs
                    .values()
                    .map(|dir| dir.at.replace("/", "_"))
                    .collect::<Vec<AutoStr>>();
                println!("GHS Location: {}", ghs_loc);
                // let relative_loc = AutoPath::new(ghs_loc.clone()).reverse_relative();
                //

                let mold_name = format!("{}.gpj", lib.local_name());
                if lib.kind == TargetKind::Device {
                    println!("MOLDNAME: {}", mold_name);
                }
                let lib_mold = Mold::new(mold_name, GHSTemplate::subproject()?);
                let mut lib_node = lib.to_node();
                lib_node.set_prop("libs", dep_names.clone());
                lib_node.set_prop("dir_names", dir_names.clone());
                lib_node.set_prop("rel", rel.clone());
                lib_node.set_prop("device_dir", device_dir.clone());
                lib_node.set_prop("incs", all_incs.clone());
                lib_node.set_prop("device_inc", device_inc.clone());
                let atom = Atom::node(lib_node);
                let lib_gen = OneGen::new(lib_mold, atom).out(AutoPath::new(ghs_loc.clone()));
                self.apps.push(lib_gen);

                for (_key, dir) in &lib.dirs {
                    println!("setting up dir {} for lib {}", dir.name, lib.name);
                    self.setup_dir(ghs_loc.clone(), dir)?;
                }
            }
        }

        Ok(())
    }

    fn finish(&mut self, _pac: &Pac) -> AutoResult<()> {
        self.gen.gen_all();
        for app in &self.apps {
            app.gen()?;
        }

        for sub in &self.subprojects {
            sub.gen()?;
        }
        Ok(())
    }
}

impl Exporter for GHSExporter {
    fn export(&mut self, pac: &mut Pac) -> AutoResult<()> {
        self.setup(pac)?;
        self.finish(pac)
    }

    fn enable_memory_output(&mut self) -> AutoResult<()> {
        Err("GHS memory output not yet implemented".into())
    }

    fn get_memory_output(&self) -> std::collections::HashMap<String, Vec<u8>> {
        std::collections::HashMap::new()
    }
}
