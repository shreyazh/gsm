use anyhow::{bail, Context, Result};
use std::process::Command;

#[derive(Debug, Clone)]
pub struct Stash {
    pub index: usize,
    pub name: String,       // e.g. "stash@{0}"
    pub message: String,    // e.g. "WIP on main: abc123 Some commit"
    pub branch: String,     // extracted branch name
    pub short_msg: String,  // user-friendly short message
    pub date: String,       // relative date from git
}

/// Ensure we are inside a git repository
pub fn assert_git_repo() -> Result<()> {
    let status = Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .output()
        .context("Failed to run git. Is git installed?")?;

    if !status.status.success() {
        bail!("Not inside a git repository. Please run gsm from within a git repo.");
    }
    Ok(())
}

/// List all stashes
pub fn list_stashes() -> Result<Vec<Stash>> {
    let output = Command::new("git")
        .args([
            "stash",
            "list",
            "--format=%gd|%gs|%cr", // stash@{N}|message|relative date
        ])
        .output()
        .context("Failed to run git stash list")?;

    let stdout = String::from_utf8_lossy(&output.stdout);

    let mut stashes = Vec::new();
    for (i, line) in stdout.lines().enumerate() {
        let parts: Vec<&str> = line.splitn(3, '|').collect();
        if parts.len() < 3 {
            continue;
        }

        let name = parts[0].to_string();
        let message = parts[1].to_string();
        let date = parts[2].to_string();

        // Extract branch from "WIP on <branch>: ..." or "On <branch>: ..."
        let branch = if message.starts_with("WIP on ") {
            message
                .strip_prefix("WIP on ")
                .and_then(|s| s.split(':').next())
                .unwrap_or("unknown")
                .trim()
                .to_string()
        } else if message.starts_with("On ") {
            message
                .strip_prefix("On ")
                .and_then(|s| s.split(':').next())
                .unwrap_or("unknown")
                .trim()
                .to_string()
        } else {
            "unknown".to_string()
        };

        // Short message: after the colon
        let short_msg = message
            .splitn(2, ": ")
            .nth(1)
            .unwrap_or(&message)
            .to_string();

        stashes.push(Stash {
            index: i,
            name,
            message,
            branch,
            short_msg,
            date,
        });
    }

    Ok(stashes)
}

/// Get the diff for a specific stash
pub fn stash_diff(stash_name: &str) -> Result<String> {
    let output = Command::new("git")
        .args(["stash", "show", "-p", "--color=never", stash_name])
        .output()
        .context("Failed to get stash diff")?;

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Get the list of files changed in a stash
pub fn stash_files(stash_name: &str) -> Result<String> {
    let output = Command::new("git")
        .args(["stash", "show", "--stat", "--color=never", stash_name])
        .output()
        .context("Failed to get stash file list")?;

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Apply a stash (keep it in the list)
pub fn apply_stash(stash_name: &str) -> Result<String> {
    let output = Command::new("git")
        .args(["stash", "apply", stash_name])
        .output()
        .context("Failed to apply stash")?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        bail!(
            "Failed to apply stash: {}",
            String::from_utf8_lossy(&output.stderr)
        )
    }
}

/// Pop a stash (apply and remove)
pub fn pop_stash(stash_name: &str) -> Result<String> {
    let output = Command::new("git")
        .args(["stash", "pop", stash_name])
        .output()
        .context("Failed to pop stash")?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        bail!(
            "Failed to pop stash: {}",
            String::from_utf8_lossy(&output.stderr)
        )
    }
}

/// Drop (delete) a stash
pub fn drop_stash(stash_name: &str) -> Result<()> {
    let output = Command::new("git")
        .args(["stash", "drop", stash_name])
        .output()
        .context("Failed to drop stash")?;

    if output.status.success() {
        Ok(())
    } else {
        bail!(
            "Failed to drop stash: {}",
            String::from_utf8_lossy(&output.stderr)
        )
    }
}

/// Create a new stash with a custom message
pub fn push_stash(message: &str, include_untracked: bool) -> Result<()> {
    let mut args = vec!["stash", "push", "-m", message];
    if include_untracked {
        args.push("--include-untracked");
    }

    let output = Command::new("git")
        .args(&args)
        .output()
        .context("Failed to push stash")?;

    if output.status.success() {
        Ok(())
    } else {
        bail!(
            "Failed to create stash: {}",
            String::from_utf8_lossy(&output.stderr)
        )
    }
}

/// Get current branch name
pub fn current_branch() -> Result<String> {
    let output = Command::new("git")
        .args(["branch", "--show-current"])
        .output()
        .context("Failed to get current branch")?;

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}