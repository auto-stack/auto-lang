//! cp command - Copy files and directories
//!
//! Provides cross-platform file copying with progress reporting.

use crate::cmd::{Command, PipelineData, Signature};
use crate::shell::Shell;
use auto_val::Value;
use miette::{IntoDiagnostic, Result};
use std::fs;
use std::path::{Path, PathBuf};

pub struct CpCommand;

impl Command for CpCommand {
    fn name(&self) -> &str {
        "cp"
    }

    fn signature(&self) -> Signature {
        Signature::new("cp", "Copy files and directories")
            .required("source", "Source file or directory")
            .required("dest", "Destination path")
            .flag_with_short("recursive", 'r', "Copy directories recursively")
            .flag_with_short("force", 'f', "Force overwrite without prompting")
            .flag_with_short("preserve", 'p', "Preserve file attributes")
            .flag("verbose", "Show what files are being copied")
    }

    fn run(
        &self,
        args: &crate::cmd::parser::ParsedArgs,
        _input: PipelineData,
        shell: &mut Shell,
    ) -> Result<PipelineData> {
        if args.positionals.len() < 2 {
            miette::bail!("cp: missing source or destination argument");
        }

        let source = args.positionals.get(0).map(|s| s.as_str()).unwrap_or(".");
        let dest = args.positionals.get(1).map(|s| s.as_str()).unwrap_or(".");

        let recursive = args.has_flag("recursive");
        let force = args.has_flag("force");
        let preserve = args.has_flag("preserve");
        let verbose = args.has_flag("verbose");

        let source_path = if Path::new(source).is_absolute() {
            PathBuf::from(source)
        } else {
            shell.pwd().join(source)
        };

        let dest_path = if Path::new(dest).is_absolute() {
            PathBuf::from(dest)
        } else {
            shell.pwd().join(dest)
        };

        if !source_path.exists() {
            miette::bail!("cp: cannot stat '{}': No such file or directory", source);
        }

        let copied = if source_path.is_dir() {
            if recursive {
                copy_dir(&source_path, &dest_path, force, preserve, verbose)?
            } else {
                miette::bail!("cp: omitting directory '{}'", source);
            }
        } else {
            copy_file(&source_path, &dest_path, force, preserve, verbose)?
        };

        // Return summary as Value
        let mut result = auto_val::Obj::new();
        result.set("source", Value::str(source));
        result.set("destination", Value::str(dest));
        result.set("files_copied", Value::I64(copied as i64));
        result.set("success", Value::Bool(true));

        Ok(PipelineData::from_value(Value::Obj(result)))
    }
}

/// Copy a single file
fn copy_file(
    source: &Path,
    dest: &Path,
    force: bool,
    preserve: bool,
    verbose: bool,
) -> Result<usize> {
    // Check if destination exists and force flag
    if dest.exists() && !force {
        miette::bail!("cp: '{}' already exists (use -f to force overwrite)", dest.display());
    }

    // If destination is a directory, place file inside it
    let final_dest = if dest.is_dir() {
        dest.join(source.file_name().unwrap_or_default())
    } else {
        dest.to_path_buf()
    };

    if verbose {
        eprintln!("'{}' -> '{}'", source.display(), final_dest.display());
    }

    // Copy the file
    if preserve {
        // Preserve permissions and timestamps
        fs::copy(source, &final_dest).into_diagnostic()?;
        copy_permissions(source, &final_dest)?;
    } else {
        fs::copy(source, &final_dest).into_diagnostic()?;
    }

    Ok(1)
}

/// Copy a directory recursively
fn copy_dir(
    source: &Path,
    dest: &Path,
    force: bool,
    preserve: bool,
    verbose: bool,
) -> Result<usize> {
    let mut count = 0;

    // Create destination directory if it doesn't exist
    if !dest.exists() {
        fs::create_dir_all(dest).into_diagnostic()?;
        if preserve {
            copy_permissions(source, dest)?;
        }
    }

    // Handle case where dest is a directory
    let base_dest = if dest.is_dir() {
        dest.join(source.file_name().unwrap_or_default())
    } else {
        dest.to_path_buf()
    };

    if !base_dest.exists() {
        fs::create_dir_all(&base_dest).into_diagnostic()?;
        if preserve {
            copy_permissions(source, &base_dest)?;
        }
    }

    // Copy all entries
    for entry in fs::read_dir(source).into_diagnostic()? {
        let entry = entry.into_diagnostic()?;
        let source_path = entry.path();
        let file_name = entry.file_name();
        let dest_path = base_dest.join(&file_name);

        if source_path.is_dir() {
            count += copy_dir(&source_path, &dest_path, force, preserve, verbose)?;
        } else {
            if verbose {
                eprintln!("'{}' -> '{}'", source_path.display(), dest_path.display());
            }

            if dest_path.exists() && !force {
                miette::bail!("cp: '{}' already exists (use -f to force overwrite)", dest_path.display());
            }

            fs::copy(&source_path, &dest_path).into_diagnostic()?;

            if preserve {
                copy_permissions(&source_path, &dest_path)?;
            }

            count += 1;
        }
    }

    Ok(count)
}

/// Copy file permissions (Unix only)
#[cfg(unix)]
fn copy_permissions(source: &Path, dest: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let perms = fs::metadata(source).into_diagnostic()?.permissions();
    fs::set_permissions(dest, perms).into_diagnostic()?;
    Ok(())
}

/// Copy file permissions (Windows - no-op)
#[cfg(windows)]
fn copy_permissions(_source: &Path, _dest: &Path) -> Result<()> {
    // Windows permission handling is more complex, skip for now
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cp_command_name() {
        let cmd = CpCommand;
        assert_eq!(cmd.name(), "cp");
    }

    #[test]
    fn test_cp_signature() {
        let cmd = CpCommand;
        let sig = cmd.signature();
        assert_eq!(sig.name, "cp");
        assert_eq!(sig.description, "Copy files and directories");
        assert_eq!(sig.arguments.iter().filter(|a| a.required).count(), 2);
    }
}
