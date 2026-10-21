use auto_val::AutoPath;
use log::{error, info};

use crate::AutoResult;

pub fn check_git(dir: &AutoPath) -> AutoResult<()> {
    if !dir.is_dir() {
        return Err(format!("Target location {} is not a dir", dir).into());
    }
    let gitdir = dir.join(".git");
    if !gitdir.is_dir() {
        return Err(format!("Target location {} is not a git dir", dir).into());
    }
    Ok(())
}

/// 经统一 owned 设施执行 git（PLAN-816 T-03 迁移点：显式 cwd/env、
/// 捕获分离、退出具名分类；git.rs 四站点）。
fn git_capture<I, S>(
    args: I,
    cwd: &std::path::Path,
) -> AutoResult<auto_tool_execution::ExecutionResult>
where
    I: IntoIterator<Item = S>,
    S: Into<std::ffi::OsString>,
{
    let mut spec = auto_tool_execution::ExecutionSpec::new({
        let mut argv = vec![std::ffi::OsString::from("git")];
        argv.extend(args.into_iter().map(Into::into));
        argv
    });
    spec.cwd = Some(cwd.to_path_buf());
    spec.tree_scope = auto_tool_execution::TreeScope::DirectChildOnly;
    auto_tool_execution::run(&spec).map_err(|e| format!("git spawn: {e}").into())
}

pub fn check_detached(dir: &AutoPath) -> AutoResult<()> {
    check_git(dir)?;
    // check if there is any modifications to this git repository
    let cmd = git_capture(["branch", "--show-current"], dir.path())?;
    if cmd.kind != auto_tool_execution::ExitKind::Success {
        return Err(format!(
            "Failed to run git branch --show-current: {}",
            String::from_utf8_lossy(cmd.stderr.as_deref().unwrap_or_default())
        )
        .into());
    }
    // `git branch --show-current`：attached 分支输出非空，detached HEAD 输出空。
    if cmd.stdout.as_deref().unwrap_or_default().is_empty() {
        return Err(format!("Dir {} is detached", dir).into());
    }
    Ok(())
}

pub fn check_changes(dir: &AutoPath) -> AutoResult<()> {
    check_git(dir)?;
    // check if there is any modifications to this git repository
    let cmd = git_capture(["status", "-s"], dir.path())?;
    if cmd.kind != auto_tool_execution::ExitKind::Success {
        return Err(format!(
            "Failed to run git status: {}",
            String::from_utf8_lossy(cmd.stderr.as_deref().unwrap_or_default())
        )
        .into());
    }
    if !cmd.stdout.as_deref().unwrap_or_default().is_empty() {
        println!(
            "\n--- git status ---\n\n{}",
            String::from_utf8_lossy(cmd.stdout.as_deref().unwrap_or_default())
        );
        return Err(format!("Target location {} has uncommitted changes", dir).into());
    }
    Ok(())
}

pub fn pull(dir: &AutoPath) -> AutoResult<()> {
    let output = git_capture(["pull"], dir.path())?;
    if output.kind == auto_tool_execution::ExitKind::Success {
        info!("updated dependency {}", dir);
        println!(
            "{}",
            String::from_utf8_lossy(output.stdout.as_deref().unwrap_or_default())
        );
    } else {
        error!("failed to update dependency {}", dir);
        println!(
            "{}",
            String::from_utf8_lossy(output.stderr.as_deref().unwrap_or_default())
        );
    }
    Ok(())
}

pub fn switch_to_master(dir: &AutoPath) -> AutoResult<()> {
    let output = git_capture(["checkout", "master"], dir.path())?;
    if output.kind == auto_tool_execution::ExitKind::Success {
        info!("switched to master branch in {}", dir);
        println!(
            "{}",
            String::from_utf8_lossy(output.stdout.as_deref().unwrap_or_default())
        );
    } else {
        error!("failed to switch to master branch in {}", dir);
        println!(
            "{}",
            String::from_utf8_lossy(output.stderr.as_deref().unwrap_or_default())
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use auto_val::AutoPath;

    fn init_repo(dir: &std::path::Path) -> AutoPath {
        let _ = git_capture(["init"], dir);
        let _ = git_capture(["config", "user.email", "t@example.com"], dir);
        let _ = git_capture(["config", "user.name", "t"], dir);
        std::fs::write(dir.join("f.txt"), "x").unwrap();
        let _ = git_capture(["add", "f.txt"], dir);
        let _ = git_capture(["commit", "-m", "init"], dir);
        AutoPath::new(dir.to_string_lossy().to_string())
    }

    /// F-04 四状态：attached/detached × clean/dirty（真实临时仓）。
    #[test]
    fn check_detached_attached_vs_detached_and_clean_dirty() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let path = init_repo(root);

        // attached + clean → Ok
        assert!(check_detached(&path).is_ok(), "attached 应 Ok");
        assert!(check_changes(&path).is_ok(), "clean 应 Ok");

        // attached + dirty → Err(uncommitted)
        std::fs::write(root.join("dirty.txt"), "y").unwrap();
        assert!(
            check_changes(&path).is_err(),
            "dirty 应 Err(uncommitted changes)"
        );
        let _ = git_capture(["checkout", "--", "."], root);
        let _ = std::fs::remove_file(root.join("dirty.txt"));

        // detached + clean → Err(detached)
        let _ = git_capture(["checkout", "--detach"], root);
        assert!(check_detached(&path).is_err(), "detached 应 Err(detached)");
        assert!(
            check_changes(&path).is_ok(),
            "detached+clean 的 dirty 检查仍 Ok"
        );

        // detached + dirty → Err(uncommitted)（dirty 语义独立于 attached）
        std::fs::write(root.join("dirty.txt"), "y").unwrap();
        assert!(
            check_changes(&path).is_err(),
            "detached+dirty 应 Err(uncommitted)"
        );
    }
}
