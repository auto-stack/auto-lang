//! ln command - Create links between files
//!
//! Creates hard or symbolic links. Supports force overwrite of existing links.

use crate::cmd::{Command, PipelineData, Signature};
use crate::cmd::parser::ParsedArgs;
use crate::shell::Shell;
use ash_core::pipeline::AtomPipeline;
use miette::{IntoDiagnostic, Result};
use std::path::PathBuf;

pub struct LnCommand;

impl Command for LnCommand {
    fn name(&self) -> &str {
        "ln"
    }

    fn signature(&self) -> Signature {
        Signature::new("ln", "Create links between files")
            .required("target", "Target file or directory")
            .required("link", "Name of the link to create")
            .flag_with_short("symbolic", 's', "Create a symbolic link")
            .flag_with_short("force", 'f', "Force overwrite existing link")
    }

    fn run(
        &self,
        args: &ParsedArgs,
        _input: PipelineData,
        shell: &mut Shell,
    ) -> Result<PipelineData> {
        if args.positionals.len() < 2 {
            miette::bail!("ln: missing target or link argument");
        }

        let target_arg = args.positionals.get(0).map(|s| s.as_str()).unwrap();
        let link_arg = args.positionals.get(1).map(|s| s.as_str()).unwrap();

        let symbolic = args.has_flag("symbolic");
        let force = args.has_flag("force");

        let target = if std::path::Path::new(target_arg).is_absolute() {
            PathBuf::from(target_arg)
        } else {
            shell.pwd().join(target_arg)
        };

        let link = if std::path::Path::new(link_arg).is_absolute() {
            PathBuf::from(link_arg)
        } else {
            shell.pwd().join(link_arg)
        };

        // Check target exists
        if !target.exists() && !symbolic {
            miette::bail!("ln: cannot stat '{}': No such file or directory", target_arg);
        }

        // Remove existing link if force flag is set
        if link.exists() || link.is_symlink() {
            if force {
                if link.is_dir() && !link.is_symlink() {
                    miette::bail!("ln: cannot overwrite directory '{}'", link_arg);
                }
                std::fs::remove_file(&link).into_diagnostic()?;
            } else {
                miette::bail!("ln: '{}' already exists (use -f to force)", link_arg);
            }
        }

        // Create the link
        if symbolic {
            #[cfg(unix)]
            {
                std::os::unix::fs::symlink(&target, &link).into_diagnostic()?;
            }
            #[cfg(windows)]
            {
                if target.is_dir() {
                    std::os::windows::fs::symlink_dir(&target, &link).into_diagnostic()?;
                } else {
                    std::os::windows::fs::symlink_file(&target, &link).into_diagnostic()?;
                }
            }
        } else {
            // Hard link — only works for files on same filesystem
            #[cfg(unix)]
            {
                std::fs::hard_link(&target, &link).into_diagnostic()?;
            }
            #[cfg(windows)]
            {
                // On Windows, hard links require both paths to be files
                if target.is_dir() {
                    miette::bail!("ln: cannot create hard link to directory '{}'", target_arg);
                }
                std::fs::hard_link(&target, &link).into_diagnostic()?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ln_command_name() {
        let cmd = LnCommand;
        assert_eq!(cmd.name(), "ln");
    }

    #[test]
    fn test_ln_signature() {
        let cmd = LnCommand;
        let sig = cmd.signature();
        assert_eq!(sig.name, "ln");
        assert_eq!(sig.arguments.iter().filter(|a| a.required).count(), 2);
    }

    #[test]
    fn test_ln_has_symbolic_flag() {
        let cmd = LnCommand;
        let sig = cmd.signature();
        assert!(sig.arguments.iter().any(|a| a.name == "symbolic" && a.is_flag));
    }
}
