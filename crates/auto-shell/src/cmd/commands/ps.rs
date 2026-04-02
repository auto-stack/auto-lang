//! ps command - List running processes
//!
//! Uses sysinfo crate for cross-platform process listing.

use crate::cmd::{Command, PipelineData, Signature};
use crate::shell::Shell;
use auto_val::{Value, Obj};
use miette::Result;
use sysinfo::System;

pub struct PsCommand;

impl Command for PsCommand {
    fn name(&self) -> &str {
        "ps"
    }

    fn signature(&self) -> Signature {
        Signature::new("ps", "List running processes")
            .flag_with_short("long", 'l', "Show detailed process information")
            .flag_with_short("all", 'a', "Show all processes (including system)")
    }

    fn run(
        &self,
        args: &crate::cmd::parser::ParsedArgs,
        _input: PipelineData,
        _shell: &mut Shell,
    ) -> Result<PipelineData> {
        let long = args.has_flag("long");
        let _all = args.has_flag("all");

        let mut sys = System::new_all();
        sys.refresh_all();

        let mut processes: Vec<ProcessInfo> = sys.processes()
            .iter()
            .map(|(pid, process)| {
                ProcessInfo {
                    pid: pid.as_u32() as i32,
                    ppid: process.parent().map(|p| p.as_u32() as i32).unwrap_or(0),
                    name: process.name().to_string_lossy().to_string(),
                    status: format!("{:?}", process.status()),
                    cpu_usage: process.cpu_usage() as f64,
                    mem_usage: process.memory() as i64,
                    start_time: None, // sysinfo doesn't provide this directly
                    command: if long {
                        Some(process.cmd().iter()
                            .map(|s| s.to_string_lossy().to_string())
                            .collect::<Vec<_>>()
                            .join(" "))
                    } else {
                        None
                    },
                }
            })
            .collect();

        // Sort by CPU usage (highest first)
        processes.sort_by(|a, b| {
            b.cpu_usage.partial_cmp(&a.cpu_usage).unwrap_or(std::cmp::Ordering::Equal)
        });

        // Convert to Value
        let values: Vec<Value> = processes.iter().map(|p| {
            let mut obj = Obj::new();
            obj.set("pid", Value::Int(p.pid));
            obj.set("ppid", Value::Int(p.ppid));
            obj.set("name", Value::str(&p.name));
            obj.set("status", Value::str(&p.status));
            obj.set("cpu", Value::Float(p.cpu_usage));
            obj.set("mem", Value::Int(p.mem_usage as i32));

            if let Some(cmd) = &p.command {
                obj.set("command", Value::str(cmd));
            }

            Value::Obj(obj)
        }).collect();

        Ok(PipelineData::from_value(Value::Array(auto_val::Array { values })))
    }
}

struct ProcessInfo {
    pid: i32,
    ppid: i32,
    name: String,
    status: String,
    cpu_usage: f64,
    mem_usage: i64,
    #[allow(dead_code)]
    start_time: Option<chrono::DateTime<chrono::Utc>>,
    command: Option<String>,
}
