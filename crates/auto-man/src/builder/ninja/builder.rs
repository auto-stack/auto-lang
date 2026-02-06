use crate::builder::Builder;
use auto_gen::AutoGen;
use auto_val::{AutoPath, AutoStr};
use dialoguer::Select;
use log::*;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::{fs::File, io::Write};

use crate::{AutoResult, Port, TargetKind};
use crate::{Pac, Target};

// 导入ninja子模块
use super::config::{CompilerConfig, CompilerKind, ExecutableType};
use super::mapper::FlagMapper;
use super::resolver::CompilerResolver;
use super::templates::CommandTemplates;

// Wrapper struct for memory output
struct MemoryWriter {
    buffer: Vec<u8>,
}

impl MemoryWriter {
    fn new() -> Self {
        Self { buffer: Vec::new() }
    }
}

impl Write for MemoryWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.buffer.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

pub struct NinjaBuilder {
    pub gen: AutoGen,
    pub path: AutoPath,
    pub out: Box<dyn Write + 'static>,
    pub compiler: Option<CompilerConfig>,
    memory_mode: bool,
    memory_output: Vec<u8>,
}

impl NinjaBuilder {
    pub fn new(path: AutoPath) -> Self {
        let gen = Self::new_gen(&path);
        Self {
            gen,
            path,
            out: Box::new(std::io::sink()),
            compiler: None,
            memory_mode: false,
            memory_output: Vec::new(),
        }
    }

    fn new_gen(path: &AutoPath) -> AutoGen {
        AutoGen::new().out(path.clone()).note('$').rename(true)
    }

    /// 从 Port 加载编译器配置
    fn load_compiler_config(&mut self, port: &Port, _pac: &Pac) -> AutoResult<()> {
        // 直接使用 port.compiler
        if let Some(compiler_config) = &port.compiler {
            info!("Using compiler config: {} from port", compiler_config.name);
            self.compiler = Some(compiler_config.clone());
            Ok(())
        } else {
            // 如果没有配置编译器，使用默认配置
            warn!("No compiler configured for port {}, using default", port.name);
            let default_config = CompilerConfig::gcc_default();
            self.compiler = Some(default_config);
            Ok(())
        }
    }

    fn reset_ninja_file(&mut self) -> AutoResult<()> {
        if self.memory_mode {
            // In memory mode, use a MemoryWriter
            self.memory_output = Vec::new();
            let writer: Box<dyn Write + 'static> = Box::new(MemoryWriter::new());
            self.out = writer;
            Ok(())
        } else {
            // check if path exists
            if Path::new(self.path.path()).is_file() {
                // remove file
                std::fs::remove_file(self.path.path())?;
            }
            let parent = self.path.parent();
            if !parent.is_dir() {
                std::fs::create_dir_all(parent.path())?;
            }
            let out = File::create(self.path.path())?;
            let writer: Box<dyn Write + 'static> = Box::new(out);
            self.out = writer;
            Ok(())
        }
    }

    fn build_source(&mut self, target: &Target, objs: &mut Vec<AutoStr>) -> AutoResult<()> {
        // 使用编译器配置获取对象文件扩展名
        let compiler = self.compiler.as_ref().expect("Compiler config should be loaded");
        let obj_ext = compiler.get_object_extension();

        // build rule for each source file
        for src in target.srcs.iter() {
            println!("Checking src: {}", src);
            if src.ends_with(".c") {
                let ofile = AutoPath::new(src).filename().replace(".c", obj_ext);
                let path = if src.starts_with("/") {
                    src
                } else {
                    &format!("{}", src).into()
                };
                println!("add build rule for {}", src);
                self.out
                    .write(format!("build {}: cc ../{}\n", ofile, path).as_bytes())?;
                objs.push(ofile);
            }
        }
        Ok(())
    }

    /// 构建include标志
    fn build_includes(_compiler: &CompilerConfig, paths: Vec<AutoStr>) -> String {
        let includes: Vec<String> = paths.iter()
            .map(|p| format!("-I../{}", AutoPath::new(p.as_str())))
            .collect();
        includes.join(" ")
    }

    /// 构建define标志
    fn build_defines(compiler: &CompilerConfig, defines: &[AutoStr]) -> String {
        FlagMapper::format_defines(compiler, defines)
    }

    /// 构建编译标志
    fn build_cflags(compiler: &CompilerConfig) -> String {
        let flags = compiler.default_cflags.iter()
            .map(|f| f.as_str())
            .collect::<Vec<_>>()
            .join(" ");

        // 添加编译器特定的额外标志
        match compiler.kind {
            CompilerKind::IAR => flags,
            _ => flags,
        }
    }
}

impl Builder for NinjaBuilder {
    fn build(&mut self, pac: &mut Pac) -> AutoResult<()> {
        self.setup(pac)?;
        let mut targets_done = HashSet::new();
        for t in &pac.targets {
            if !t.deps.is_empty() {
                for dep in t.deps.iter() {
                    if targets_done.contains(&dep.rename) {
                        continue;
                    } else {
                        self.target(dep, pac)?;
                        targets_done.insert(dep.rename.clone());
                    }
                }
            }
            if targets_done.contains(&t.rename) {
                continue;
            } else {
                self.target(t, pac)?;
                targets_done.insert(t.rename.clone());
            }
        }
        self.finish(pac)?;
        Ok(())
    }

    fn setup(&mut self, pac: &mut Pac) -> AutoResult<()> {
        // load srcs in sub dirs into targets
        pac.collect_srcs()?;

        // 加载编译器配置
        self.load_compiler_config(&pac.port, pac)?;

        // 准备所有需要的数据（避免借用冲突）
        let all_incs = pac.all_incs();
        let defines = pac.defines.clone();
        let pac_name = pac.name.clone();

        // 获取编译器配置的克隆，以便在 reset_ninja_file 之外使用
        let compiler = self.compiler.as_ref().expect("Compiler config should be loaded").clone();

        // 解析编译器路径
        let cc_path = CompilerResolver::resolve_executable(&compiler, ExecutableType::Compiler);
        let as_path = CompilerResolver::resolve_executable(&compiler, ExecutableType::Assembler);
        let link_path = CompilerResolver::resolve_executable(&compiler, ExecutableType::Linker);
        let ar_path = CompilerResolver::resolve_executable(&compiler, ExecutableType::Archiver);

        // 打印调试信息（在move之前）
        println!("Got includes: {:?}", all_incs);

        // setup Ninja file header（在最后，避免借用冲突）
        self.reset_ninja_file()?;

        // 构建标志（all_incs在这里被move）
        let includes = Self::build_includes(&compiler, all_incs);
        let defines_flags = Self::build_defines(&compiler, &defines);
        let cflags = Self::build_cflags(&compiler);

        // 获取命令模板
        let templates = CommandTemplates::new(&compiler);

        // 渲染命令
        let compile_only = FlagMapper::map_compile_only(&compiler);
        let asm_cmd = templates.render_assemble(
            as_path.to_astr().as_str(),
            &includes,
            &cflags,
            "$out",
            "$in"
        );
        let cc_cmd = templates.render_compile(
            cc_path.to_astr().as_str(),
            &compile_only,
            &includes,
            &defines_flags,
            &cflags,
            "$out",
            "$in"
        );
        let link_cmd = templates.render_link(
            link_path.to_astr().as_str(),
            "$out",
            "$in",
            "$ldflags",
            "$libs"
        );
        let lib_cmd = templates.render_archive(
            ar_path.to_astr().as_str(),
            "$out",
            "$in"
        );

        // 写入build.ninja头部
        self.out.write(
            format!(
                r#"# ninja build file for {} with {} compiler

# includes
includes = {}

# defines
defines = {}

# cflags
cflags = {}

# rules
rule as
    command = {}
    description = Assembling $in

rule cc
    command = {}
    description = Compiling $in

rule link
    command = {}
    description = Linking $out

rule lib
    command = {}
    description = Building library $out

"#,
                pac_name,
                compiler.name,
                includes,
                defines_flags,
                cflags,
                asm_cmd,
                cc_cmd,
                link_cmd,
                lib_cmd
            )
            .as_bytes(),
        )?;

        Ok(())
    }

    fn target(&mut self, target: &Target, pac: &Pac) -> AutoResult<()> {
        println!("[ninja] building target {}", target.name);
        self.out
            .write(format!("# build objects for target {}\n", target.name).as_bytes())?;

        let mut objs: Vec<AutoStr> = Vec::new();
        self.build_source(target, &mut objs)?;

        let out = self.out.as_mut();
        // deal with the main target of this project
        match target.kind {
            TargetKind::App | TargetKind::Test => {
                let n = if target.name.is_empty() {
                    match target.kind {
                        TargetKind::App => "main".into(),
                        TargetKind::Test => "test".into(),
                        _ => unreachable!(),
                    }
                } else {
                    target.name.clone()
                };

                let target_name = match target.port.platform.as_str() {
                    "windows" => format!("{}.exe", n).into(),
                    "linux" => n.clone(),
                    _ => n.clone(),
                };

                out.write(b"# target executable\n")?;
                out.write(format!("build {}: link {}", target_name, objs.join(" ")).as_bytes())?;

                // add dependency libs
                let deps = target
                    .links
                    .iter()
                    .map(|d| {
                        let n = d.main_arg().repr();
                        if let Some(t) = pac.get_target(&n) {
                            t.libname().to_string()
                        } else {
                            "".to_string()
                        }
                    })
                    .filter(|t| !t.is_empty())
                    .collect::<Vec<String>>()
                    .join(" ");
                if !deps.is_empty() {
                    out.write(b" ")?;
                    out.write(deps.as_bytes())?;
                }
                out.write(b"\n")?;
            }
            TargetKind::Lib | TargetKind::Dep | TargetKind::Device => {
                let target_name: AutoStr = target.libname();
                let objs = objs.join(" ");
                if !objs.is_empty() {
                    out.write(b"# target library\n")?;
                    out.write(format!("build {}: lib {}\n", target_name, objs).as_bytes())?;
                    out.write(b"\n")?;
                }
            }
            TargetKind::Bag => {
                println!("DOING WITH BAG: {}", target.name);
            }
        }
        // out.write(format!("build {}.o: cc ../{}.c\n", target.name, target.name).as_bytes())?;

        let is_header_only = target.srcs.is_empty() && !target.incs.is_empty();
        // warn!("Target [{}] srcs: {:?}", target.name, target.srcs);

        // out.write(format!("{}({} ", cmd, target.name).as_bytes())?;
        if is_header_only {
            // out.write(b"INTERFACE")?;
        } else {
            // print_list(&target.srcs, out)?;
        }
        // out.write(b")\n\n")?;

        let incs = target
            .incs
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join(" ");
        if !incs.is_empty() {
            // out.write(format!("target_include_directories({} {} ", target.name, attr).as_bytes())?;
            // print_list(&target.incs, out)?;
            // out.write(b")\n")?;
        }

        // defines
        if !target.defines.is_empty() {
            // out.write(format!("target_compile_definitions({} {} ", target.name, attr).as_bytes())?;
            // print_vec(&target.defines, out)?;
            // out.write(b")\n")?;
        }

        if target.links.len() > 0 {}

        // out.write(b"\n")?;

        Ok(())
    }

    fn finish(&mut self, _pac: &Pac) -> AutoResult<()> {
        let out = self.out.as_mut();
        out.write(b"\n")?;
        out.flush()?;

        // In memory mode, extract data from MemoryWriter
        if self.memory_mode {
            // Use unsafe to extract the MemoryWriter data
            let raw_ptr = self.out.as_ref() as *const dyn Write as *const MemoryWriter;
            if !raw_ptr.is_null() {
                unsafe {
                    let writer = &*raw_ptr;
                    self.memory_output = writer.buffer.clone();
                }
            }
            self.out = Box::new(std::io::sink());
            return Ok(());
        }

        // let out be finished and close the file
        self.out = Box::new(std::io::sink());
        let build_path = self.path.parent();

        println!(
            "[ninja] calling ninja to build executable in {}",
            build_path
        );

        // run ninja to build
        let mut child = std::process::Command::new("ninja")
            .args(["-C", build_path.to_astr().as_str()])
            .spawn()
            .expect("Failed to spawn ninja process");

        let status = child.wait().expect("Failed to wait for ninja process");

        println!("ninja build finished with status: {}", status);

        println!("End of build");
        Ok(())
    }

    fn clean(&mut self) -> AutoResult<()> {
        // remove NinjaLists.txt
        std::fs::remove_file(&self.path.path())?;
        // remove build directory
        std::fs::remove_dir_all("build")?;

        // Ninja
        let files = glob::glob("NinjaLists*")?;
        for file in files {
            if let Ok(file) = file {
                info!("deleting file {}", file.display());
                std::fs::remove_file(file)?;
            }
        }

        Ok(())
    }

    fn run(&mut self, pac: &Pac, args: Vec<String>) -> AutoResult<()> {
        // find build directry
        let build_dir = pac.port.at.clone();
        if !AutoPath::new(&build_dir).exists() {
            return Err("Build directory does not exist".into());
        }

        // look for .exe in build directory
        let mut exes = Vec::new();
        for entry in std::fs::read_dir(build_dir.to_string())? {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                let n = entry.file_name().into_string().unwrap();
                if is_executable::is_executable(&entry.path()) {
                    exes.push(format!("{}/{}", build_dir, n));
                }
            }
        }
        // let user select one to run
        if exes.is_empty() {
            return Err("No executable found".into());
        } else if exes.len() == 1 {
            let target = exes[0].clone();
            println!("[ninja] running exe: {}", target);
            let status = std::process::Command::new(target).args(args).status()?;
            if !status.success() {
                return Err(format!("Failed to run Ninja").into());
            }
        } else {
            // TODO: implement user selection
            let selection = Select::new()
                .with_prompt("Which executable do you want to run?")
                .default(0)
                .items(&exes)
                .interact()?;

            let target = exes[selection].clone();
            println!("[ninja] running exe: {}", target);
            let status = std::process::Command::new(target).args(args).status()?;
            if !status.success() {
                return Err(format!("Failed to run Ninja").into());
            }
        }
        Ok(())
    }

    fn enable_memory_output(&mut self) -> AutoResult<()> {
        self.memory_mode = true;
        self.memory_output = Vec::new();
        Ok(())
    }

    fn get_memory_output(&self) -> HashMap<String, Vec<u8>> {
        let mut map = HashMap::new();
        if self.memory_mode {
            // Extract filename from path
            let filename = Path::new(self.path.path())
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("build.ninja");
            map.insert(filename.to_string(), self.memory_output.clone());
        }
        map
    }
}
