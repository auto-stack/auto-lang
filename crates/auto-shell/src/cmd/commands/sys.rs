//! sys command - System information
//!
//! Provides disk, cpu, and memory information using sysinfo crate.

use crate::cmd::{Command, PipelineData, Signature};
use crate::shell::Shell;
use auto_val::{Value, Obj};
use miette::Result;
use sysinfo::{System, Disks, CpuRefreshKind};

pub struct SysCommand;

impl Command for SysCommand {
    fn name(&self) -> &str {
        "sys"
    }

    fn signature(&self) -> Signature {
        Signature::new("sys", "Get system information")
            .optional("subcommand", "Subcommand: disks, cpu, mem")
    }

    fn run(
        &self,
        args: &crate::cmd::parser::ParsedArgs,
        _input: PipelineData,
        _shell: &mut Shell,
    ) -> Result<PipelineData> {
        let subcommand = args.positionals.get(0).map(|s| s.as_str()).unwrap_or("all");

        match subcommand {
            "disks" => sys_disks(),
            "cpu" => sys_cpu(),
            "mem" | "memory" => sys_mem(),
            "all" => sys_all(),
            _ => miette::bail!("sys: unknown subcommand '{}'. Use: disks, cpu, mem", subcommand),
        }
    }
}

fn sys_disks() -> Result<PipelineData> {
    let disks = Disks::new_with_refreshed_list();

    let values: Vec<Value> = disks.iter().map(|disk| {
        let mut obj = Obj::new();
        obj.set("device", Value::str(&disk.name().to_string_lossy().to_string()));
        obj.set("type", Value::str(&disk.file_system().to_string_lossy().to_string()));
        obj.set("mount", Value::str(&disk.mount_point().to_string_lossy().to_string()));
        obj.set("total", Value::Int(disk.total_space() as i32));
        obj.set("free", Value::Int(disk.available_space() as i32));
        obj.set("removable", Value::Bool(disk.is_removable()));
        Value::Obj(obj)
    }).collect();

    Ok(PipelineData::from_value(Value::Array(auto_val::Array { values })))
}

fn sys_cpu() -> Result<PipelineData> {
    let mut sys = System::new();
    sys.refresh_cpu_specifics(CpuRefreshKind::everything());

    let cpus: Vec<Value> = sys.cpus().iter().enumerate().map(|(i, cpu)| {
        let mut obj = Obj::new();
        obj.set("index", Value::Int(i as i32));
        obj.set("name", Value::str(cpu.name()));
        obj.set("vendor", Value::str(cpu.vendor_id()));
        obj.set("brand", Value::str(cpu.brand()));
        obj.set("frequency", Value::Int(cpu.frequency() as i32));
        obj.set("usage", Value::Float(cpu.cpu_usage() as f64));
        Value::Obj(obj)
    }).collect();

    Ok(PipelineData::from_value(Value::Array(auto_val::Array { values: cpus })))
}

fn sys_mem() -> Result<PipelineData> {
    let mut sys = System::new();
    sys.refresh_memory();

    let total = sys.total_memory() as i64;
    let free = sys.free_memory() as i64;
    let available = sys.available_memory() as i64;
    let used = total.saturating_sub(available);
    let usage_percent = if total > 0 { (used as f64 / total as f64) * 100.0 } else { 0.0 };

    let mut obj = Obj::new();
    obj.set("total", Value::I64(total));
    obj.set("free", Value::I64(free));
    obj.set("available", Value::I64(available));
    obj.set("used", Value::I64(used));
    obj.set("usage_percent", Value::Float(usage_percent));

    Ok(PipelineData::from_value(Value::Obj(obj)))
}

fn sys_all() -> Result<PipelineData> {
    let mut obj = Obj::new();

    // Get disks
    let disks = Disks::new_with_refreshed_list();
    let disk_values: Vec<Value> = disks.iter().map(|disk| {
        let mut d = Obj::new();
        d.set("device", Value::str(&disk.name().to_string_lossy().to_string()));
        d.set("mount", Value::str(&disk.mount_point().to_string_lossy().to_string()));
        d.set("total", Value::Int(disk.total_space() as i32));
        d.set("free", Value::Int(disk.available_space() as i32));
        Value::Obj(d)
    }).collect();
    obj.set("disks", Value::Array(auto_val::Array { values: disk_values }));

    // Get memory
    let mut sys = System::new();
    sys.refresh_memory();
    obj.set("total_memory", Value::Int(sys.total_memory() as i32));
    obj.set("free_memory", Value::Int(sys.free_memory() as i32));

    Ok(PipelineData::from_value(Value::Obj(obj)))
}
