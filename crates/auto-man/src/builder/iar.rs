#![allow(deprecated)]
use super::Builder;
use crate::asset::IARTemplate;
use crate::target::{Target, TargetKind};
use crate::AutoResult;
use crate::Pac; // Changed from crate::pac::Pac (pac.rs not migrated yet)
use auto_gen::*;
use auto_val::AutoPath;
use log::*;

pub struct IARBuilder {
    pub gen: AutoGen,
    pub app_gens: Vec<AutoGen>,
    pub path: AutoPath,
}

impl IARBuilder {
    pub fn new(path: AutoPath) -> Self {
        let gen = Self::new_gen(&path);
        Self {
            gen,
            app_gens: Vec::new(),
            path,
        }
    }

    fn new_gen(path: &AutoPath) -> AutoGen {
        AutoGen::new().out(path.clone()).note('@').rename(true)
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

impl Builder for IARBuilder {
    fn build(&mut self, pac: &mut Pac) -> AutoResult<()> {
        self.setup(pac)?;
        self.finish(pac)
    }

    fn setup(&mut self, pac: &mut Pac) -> AutoResult<()> {
        // check devices
        self.check_devices(pac)?;
        // check necessary configs
        if !pac.has_device_prop("icf") {
            error!("icf file location is not set");
            return Err("Please set the icf file location in the device file".into());
        }
        if !pac.has_device_prop("ddf") {
            error!("ddf file location is not set");
            return Err("Please set the ddf file location in the device file".into());
        }
        if !pac.has_device_prop("board") {
            error!("board file is not set");
            return Err("Please set the board file in the device file".into());
        }
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
        gen = gen.data(pac.to_atom());
        let molds = vec![Mold::new("iar.eww", IARTemplate::eww()?)];
        gen = gen.molds(molds);
        self.gen = gen;

        let mut app_gens = Vec::new();

        for app in &pac.build_targets_mut() {
            let mut app_gen = Self::new_gen(&self.path);
            app_gen = app_gen.data(app.to_atom());
            let app_molds = vec![
                Mold::new("iar.ewp", IARTemplate::ewp()?),
                Mold::new("iar.ewt", IARTemplate::ewt()?),
                Mold::new("iar.ewd", IARTemplate::ewd()?),
            ];
            app_gen = app_gen.molds(app_molds);
            app_gens.push(app_gen);
        }

        self.app_gens = app_gens;
        Ok(())
    }

    fn finish(&mut self, _pac: &Pac) -> AutoResult<()> {
        self.gen.gen_all();
        for g in &self.app_gens {
            g.gen_all();
        }
        Ok(())
    }

    fn target(&mut self, _t: &Target, _pac: &Pac) -> AutoResult<()> {
        Ok(())
    }

    fn clean(&mut self) -> AutoResult<()> {
        if self.path.is_dir() {
            info!("deleting directory {}", self.path);
            self.path.clean_with_parents()?;
            // std::fs::remove_dir_all(self.path.path())?;
        } else {
            info!("build directory {} does not exist, skipping ...", self.path);
        }

        Ok(())
    }

    fn run(&mut self, _pac: &Pac, _args: Vec<String>) -> AutoResult<()> {
        Ok(())
    }

    fn enable_memory_output(&mut self) -> AutoResult<()> {
        // TODO: Implement IAR memory output in Phase 3
        Err("IAR memory output not yet implemented".into())
    }

    fn get_memory_output(&self) -> std::collections::HashMap<String, Vec<u8>> {
        // TODO: Implement IAR memory output in Phase 3
        std::collections::HashMap::new()
    }
}
