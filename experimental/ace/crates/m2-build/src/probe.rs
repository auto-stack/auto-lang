//! PLAN-978 T-B0-V/T-04 (V-B0-EDIT, V-01/V-10): the INDEPENDENT toolchain
//! process oracle.
//!
//! Two observation layers, deliberately separate:
//! * the driver's own spawn seam (every child process the driver launches
//!   is counted by name — a self-report, useful but not authoritative);
//! * THIS module: machine-level snapshots of cargo/rustc/bindgen/clang
//!   processes by image name (`tasklist` on Windows), taken BEFORE and
//!   AFTER an operation — the independent oracle that catches a hidden
//!   build regardless of what the driver thinks it spawned.
//!
//! Linux observation is not implemented (2026-11-15 ruling: no Linux
//! validation on this machine; owner PLAN-1063) — `available` reports it.

use std::collections::BTreeMap;

/// The toolchain images the oracle watches (rust-lld is NOT in the set:
/// the native lane links through it legitimately during real builds).
pub const TOOL_IMAGES: &[&str] = &["cargo.exe", "rustc.exe", "bindgen.exe", "clang.exe"];

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ToolProcessSnapshot {
    /// image name (lowercase) → observed count
    pub counts: BTreeMap<String, u32>,
    /// false when the platform probe is unavailable (non-Windows hosts).
    pub available: bool,
}

impl ToolProcessSnapshot {
    /// Images whose count INCREASED between `before` and `self`.
    pub fn increases_since(&self, before: &ToolProcessSnapshot) -> Vec<String> {
        let mut out = Vec::new();
        for (img, count) in &self.counts {
            let prev = before.counts.get(img).copied().unwrap_or(0);
            if *count > prev {
                out.push(format!("{img}+{}", count - prev));
            }
        }
        out
    }
}

/// Take one machine-level snapshot of toolchain processes.
#[cfg(windows)]
pub fn snapshot() -> ToolProcessSnapshot {
    let output = std::process::Command::new("tasklist")
        .args(["/FO", "CSV", "/NH"])
        .output();
    let mut counts = BTreeMap::new();
    let available = match output {
        Ok(out) if out.status.success() => {
            let text = String::from_utf8_lossy(&out.stdout);
            for line in text.lines() {
                let image = line
                    .split(',')
                    .next()
                    .unwrap_or("")
                    .trim_matches('"')
                    .to_ascii_lowercase();
                for watched in TOOL_IMAGES {
                    if image == *watched {
                        *counts.entry(image.clone()).or_insert(0u32) += 1;
                    }
                }
            }
            true
        }
        _ => false,
    };
    ToolProcessSnapshot { counts, available }
}

#[cfg(not(windows))]
pub fn snapshot() -> ToolProcessSnapshot {
    ToolProcessSnapshot {
        counts: BTreeMap::new(),
        available: false,
    }
}

/// ATTRIBUTED observation (Windows): toolchain processes whose parent
/// chain descends from `root_pid`. This is the precise "WE did not
/// launch a toolchain" oracle — a concurrent external build (another
/// agent's worktree) does not false-positive it, and a hidden build
/// inside this process tree cannot escape it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OwnedToolProcesses {
    /// image name → count, all descended from the root process
    pub counts: BTreeMap<String, u32>,
    pub available: bool,
}

#[cfg(windows)]
pub fn owned_snapshot(root_pid: u32) -> OwnedToolProcesses {
    // One CIM query: pid, name, parent — then walk chains in-process.
    let script = format!(
        "Get-CimInstance Win32_Process | Where-Object {{ $_.Name -match 'cargo|rustc|bindgen|clang' }} | ForEach-Object {{ \"$($_.ProcessId)|$($_.Name)|$($_.ParentProcessId)\" }}"
    );
    let output = std::process::Command::new("powershell")
        .args(["-NoProfile", "-Command", &script])
        .output();
    let mut rows: Vec<(u32, String, u32)> = Vec::new();
    let available = match output {
        Ok(out) if out.status.success() => {
            for line in String::from_utf8_lossy(&out.stdout).lines() {
                let parts: Vec<&str> = line.split('|').collect();
                if parts.len() == 3 {
                    if let (Ok(pid), Ok(ppid)) = (parts[0].trim().parse(), parts[2].trim().parse())
                    {
                        rows.push((pid, parts[1].trim().to_ascii_lowercase(), ppid));
                    }
                }
            }
            true
        }
        _ => false,
    };
    let by_pid: BTreeMap<u32, u32> = rows.iter().map(|(pid, _, ppid)| (*pid, *ppid)).collect();
    let mut counts = BTreeMap::new();
    for (pid, name, _) in &rows {
        // walk up (bounded — guard against pid-reuse cycles)
        let mut cur = *pid;
        for _ in 0..16 {
            if cur == root_pid {
                *counts.entry(name.clone()).or_insert(0u32) += 1;
                break;
            }
            match by_pid.get(&cur) {
                Some(parent) => cur = *parent,
                None => break,
            }
        }
    }
    OwnedToolProcesses { counts, available }
}

#[cfg(not(windows))]
pub fn owned_snapshot(_root_pid: u32) -> OwnedToolProcesses {
    OwnedToolProcesses {
        counts: BTreeMap::new(),
        available: false,
    }
}
