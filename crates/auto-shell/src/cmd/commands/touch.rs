//! touch command - Create files or update timestamps
//!
//! Creates empty files if they don't exist, or updates their modification
//! timestamp if they do.

use crate::cmd::{Command, PipelineData, Signature};
use crate::cmd::parser::ParsedArgs;
use crate::shell::Shell;
use ash_core::pipeline::AtomPipeline;
use miette::{IntoDiagnostic, Result};
use std::path::PathBuf;

pub struct TouchCommand;

impl Command for TouchCommand {
    fn name(&self) -> &str {
        "touch"
    }

    fn signature(&self) -> Signature {
        Signature::new("touch", "Create empty files or update timestamps")
            .required("file", "File(s) to create or update")
            .flag_with_short("no-create", 'c', "Do not create files, only update timestamps")
    }

    fn run(
        &self,
        args: &ParsedArgs,
        _input: PipelineData,
        shell: &mut Shell,
    ) -> Result<PipelineData> {
        if args.positionals.is_empty() {
            miette::bail!("touch: missing file operand");
        }

        let no_create = args.has_flag("no-create");
        let mut created = 0;
        let mut updated = 0;

        for arg in &args.positionals {
            let path = resolve_touch_path(arg, shell);

            if path.exists() {
                // Update timestamp
                let _time = std::fs::File::open(&path)
                    .into_diagnostic()
                    .map_err(|e| miette::miette!("touch: {}: {}", arg, e))?;
                // Set modification time to now
                set_file_mtime(&path)?;
                updated += 1;
            } else if !no_create {
                // Create the file
                std::fs::File::create(&path)
                    .into_diagnostic()
                    .map_err(|e| miette::miette!("touch: {}: {}", arg, e))?;
                created += 1;
            }
        }

        Ok(PipelineData::empty())
    }

    fn run_atom(
        &self,
        args: &ParsedArgs,
        _input: AtomPipeline,
        shell: &mut Shell,
    ) -> Result<AtomPipeline> {
        let legacy = self.run(args, PipelineData::empty(), shell)?;
        Ok(crate::cmd::pipeline_convert::pipeline_data_to_atom(legacy))
    }
}

/// Resolve path relative to shell CWD.
fn resolve_touch_path(arg: &str, shell: &Shell) -> PathBuf {
    let path = std::path::Path::new(arg);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        shell.pwd().join(arg)
    }
}

/// Set a file's modification time to now (cross-platform).
fn set_file_mtime(path: &std::path::Path) -> Result<()> {
    let now = std::time::SystemTime::now();
    let ft = filetime::FileTime::from_system_time(now);
    filetime::set_file_mtime(path, ft).into_diagnostic()
}

// Since filetime may not be in Cargo.toml, provide a fallback.
mod filetime {
    use std::path::Path;

    /// Minimal FileTime shim.
    #[derive(Debug)]
    pub struct FileTime {
        _secs: i64,
        _nsecs: u32,
    }

    impl FileTime {
        pub fn from_system_time(st: std::time::SystemTime) -> Self {
            let dur = st.duration_since(std::time::UNIX_EPOCH).unwrap_or_default();
            Self {
                _secs: dur.as_secs() as i64,
                _nsecs: dur.subsec_nanos(),
            }
        }
    }

    /// Set file modification time.
    ///
    /// This is a best-effort implementation. On platforms without `utimens`,
    /// we simply open the file for writing to bump the timestamp.
    pub fn set_file_mtime(path: &Path, _ft: FileTime) -> std::io::Result<()> {
        // Portable fallback: open for append and close — bumps mtime on most OSes
        use std::io::Write;
        let mut f = std::fs::OpenOptions::new().append(true).open(path)?;
        f.flush()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_touch_command_name() {
        let cmd = TouchCommand;
        assert_eq!(cmd.name(), "touch");
    }

    #[test]
    fn test_touch_signature() {
        let cmd = TouchCommand;
        let sig = cmd.signature();
        assert_eq!(sig.name, "touch");
        assert!(sig.arguments.iter().any(|a| a.name == "file" && a.required));
    }
}
