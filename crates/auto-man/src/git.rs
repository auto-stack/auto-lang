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

pub fn check_detached(dir: &AutoPath) -> AutoResult<()> {
    check_git(dir)?;
    // check if there is any modifications to this git repository
    let cmd = std::process::Command::new("git")
        .arg("branch")
        .arg("--show-current")
        .current_dir(&dir.path())
        .output()
        .map_err(|e| format!("Failed to run git status: {}", e))?;
    if !cmd.status.success() {
        return Err(format!(
            "Failed to run git branch --show-current: {}",
            String::from_utf8_lossy(&cmd.stderr)
        )
        .into());
    }
    if cmd.stdout.is_empty() {
        return Err(format!("Dir {} is detached", dir).into());
    }
    Ok(())
}

pub fn check_changes(dir: &AutoPath) -> AutoResult<()> {
    check_git(dir)?;
    // check if there is any modifications to this git repository
    let cmd = std::process::Command::new("git")
        .arg("status")
        .arg("-s")
        .current_dir(&dir.path())
        .output()
        .map_err(|e| format!("Failed to run git status: {}", e))?;
    if !cmd.status.success() {
        return Err(format!(
            "Failed to run git status: {}",
            String::from_utf8_lossy(&cmd.stderr)
        )
        .into());
    }
    if !cmd.stdout.is_empty() {
        println!(
            "\n--- git status ---\n\n{}",
            String::from_utf8_lossy(&cmd.stdout)
        );
        return Err(format!("Target location {} has uncommitted changes", dir).into());
    }
    Ok(())
}

pub fn pull(dir: &AutoPath) -> AutoResult<()> {
    let mut cmd = std::process::Command::new("git");
    cmd.arg("pull");
    cmd.current_dir(dir.to_string().as_str());
    let output = cmd.output()?;
    if output.status.success() {
        info!("updated dependency {}", dir);
        println!("{}", String::from_utf8_lossy(&output.stdout));
    } else {
        error!("failed to update dependency {}", dir);
        println!("{}", String::from_utf8_lossy(&output.stderr));
    }
    Ok(())
}

pub fn switch_to_master(dir: &AutoPath) -> AutoResult<()> {
    let mut cmd = std::process::Command::new("git");
    cmd.arg("checkout");
    cmd.arg("master");
    cmd.current_dir(dir.to_string().as_str());
    let output = cmd.output()?;
    if output.status.success() {
        info!("switched to master branch in {}", dir);
        println!("{}", String::from_utf8_lossy(&output.stdout));
    } else {
        error!("failed to switch to master branch in {}", dir);
        println!("{}", String::from_utf8_lossy(&output.stderr));
    }
    Ok(())
}
