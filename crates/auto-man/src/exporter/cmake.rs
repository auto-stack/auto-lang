use super::Exporter;
use auto_val::AutoStr;
use log::*;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Write;
use std::path::Path;

use crate::AutoResult;
use crate::{Pac, Target, TargetKind};

// Wrapper struct for memory output that collects data into a Vec
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

pub struct CMakeExporter {
    pub cmake_path: String,
    pub out: Box<dyn Write + 'static>,
    memory_mode: bool,
    memory_output: Vec<u8>,
}

impl CMakeExporter {
    pub fn new(path: &str) -> Self {
        Self {
            cmake_path: path.to_string(),
            out: Box::new(std::io::sink()),
            memory_mode: false,
            memory_output: Vec::new(),
        }
    }

    fn clear_cmake_file(&mut self) -> AutoResult<()> {
        if self.memory_mode {
            self.memory_output = Vec::new();
            let writer: Box<dyn Write + 'static> = Box::new(MemoryWriter::new());
            self.out = writer;
            Ok(())
        } else {
            if Path::new(&self.cmake_path).is_file() {
                std::fs::remove_file(&self.cmake_path)?;
            }
            let out = File::create(&self.cmake_path)?;
            let writer: Box<dyn Write + 'static> = Box::new(out);
            self.out = writer;
            Ok(())
        }
    }

    fn setup(&mut self, pac: &mut Pac) -> AutoResult<()> {
        // load srcs in sub dirs into targets
        pac.collect_srcs()?;

        // setup cmake file header
        self.clear_cmake_file()?;
        self.out.write(b"cmake_minimum_required(VERSION 3.22)\n")?;
        self.out
            .write(format!("project({} LANGUAGES C CXX ASM)\n", pac.name).as_bytes())?;
        self.out.write(b"\n")?;
        Ok(())
    }

    fn finish(&mut self, _pac: &Pac) -> AutoResult<()> {
        let out = self.out.as_mut();
        out.flush()?;

        if self.memory_mode {
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

        self.out = Box::new(std::io::sink());

        // We only generate the project files, no need to run cmake build here for exporters
        Ok(())
    }

    fn target(&mut self, target: &Target, _pac: &Pac) -> AutoResult<()> {
        println!("exporting target {}", target.name);
        let out = self.out.as_mut();

        let cmd = match target.kind {
            TargetKind::App => "add_executable",
            TargetKind::Bag => "add_library",
            TargetKind::Lib => "add_library",
            TargetKind::Dep => "add_library",
            TargetKind::Device => "add_library",
            TargetKind::Test => "add_executable",
        };

        let is_header_only = target.srcs.is_empty() && !target.incs.is_empty();

        out.write(format!("{}({} ", cmd, target.name).as_bytes())?;
        if is_header_only {
            out.write(b"INTERFACE")?;
        } else {
            print_list(&target.srcs, out)?;
        }
        out.write(b")\n\n")?;

        let incs = target
            .incs
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join(" ");
        if !incs.is_empty() {
            let attr = if is_header_only {
                "INTERFACE"
            } else {
                "PUBLIC"
            };
            out.write(format!("target_include_directories({} {} ", target.name, attr).as_bytes())?;
            print_list(&target.incs, out)?;
            out.write(b")\n")?;
        }

        if !target.defines.is_empty() {
            let attr = if is_header_only {
                "INTERFACE"
            } else {
                "PUBLIC"
            };
            out.write(format!("target_compile_definitions({} {} ", target.name, attr).as_bytes())?;
            print_vec(&target.defines, out)?;
            out.write(b")\n")?;
        }

        if target.links.len() > 0 {
            for link in target.links.iter() {
                out.write(
                    format!("target_link_libraries({} {})\n", target.name, link.id()).as_bytes(),
                )?;
            }
        }

        out.write(b"\n")?;

        Ok(())
    }
}

fn print_vec(list: &Vec<AutoStr>, out: &mut dyn Write) -> AutoResult<()> {
    let mut sorted_list = list.clone();
    sorted_list.sort();

    if sorted_list.len() == 1 {
        out.write(format!("{}", sorted_list[0].as_str()).as_bytes())?;
    } else {
        out.write(b"\n")?;
        for item in sorted_list.iter() {
            out.write(format!("    {}\n", item.as_str()).as_bytes())?;
        }
    }
    Ok(())
}

fn print_list(list: &HashSet<AutoStr>, out: &mut dyn Write) -> AutoResult<()> {
    let mut sorted_list = list.iter().collect::<Vec<_>>();
    sorted_list.sort();

    if sorted_list.len() == 1 {
        out.write(format!("{}", sorted_list[0].as_str()).as_bytes())?;
    } else {
        out.write(b"\n")?;
        for item in sorted_list.iter() {
            out.write(format!("    {}\n", item.as_str()).as_bytes())?;
        }
    }
    Ok(())
}

impl Exporter for CMakeExporter {
    fn export(&mut self, pac: &mut Pac) -> AutoResult<()> {
        self.setup(pac)?;
        let mut targets_done = HashSet::new();
        for t in &pac.targets {
            if targets_done.contains(&t.rename) {
                continue;
            } else {
                self.target(t, pac)?;
                targets_done.insert(t.rename.clone());
            }
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
        }
        self.finish(pac)?;
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
            let filename = Path::new(&self.cmake_path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("CMakeLists.txt");
            map.insert(filename.to_string(), self.memory_output.clone());
        }
        map
    }
}
