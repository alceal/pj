use anyhow::{anyhow, Result};
use dialoguer::Select;
use std::io::IsTerminal;
use std::path::Path;
use std::process::{Command, Stdio};

#[derive(Debug, Clone, Copy)]
pub enum Visibility {
    Public,
    Private,
}

pub fn is_interactive() -> bool {
    std::io::stdin().is_terminal() && std::io::stdout().is_terminal()
}

pub fn is_gh_installed() -> bool {
    Command::new("gh")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

pub fn is_gh_authenticated() -> bool {
    Command::new("gh")
        .args(["auth", "status"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

pub fn has_remote_origin(path: &Path) -> bool {
    Command::new("git")
        .args(["remote", "get-url", "origin"])
        .current_dir(path)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

pub fn prompt_visibility() -> Result<Visibility> {
    let items = vec!["Public", "Private"];
    let selection = Select::new()
        .with_prompt("Repository visibility")
        .items(&items)
        .default(1)
        .interact()?;

    Ok(match selection {
        0 => Visibility::Public,
        _ => Visibility::Private,
    })
}

pub fn create_github_repo(path: &Path, visibility: Visibility) -> Result<()> {
    let dirname = path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| anyhow!("Invalid directory name"))?;

    let visibility_flag = match visibility {
        Visibility::Public => "--public",
        Visibility::Private => "--private",
    };

    let output = Command::new("gh")
        .args(["repo", "create", dirname, "--source=.", visibility_flag])
        .current_dir(path)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);

        if stderr.contains("Name already exists") || stderr.contains("already exists") {
            eprintln!("Error: Repository '{}' already exists on GitHub.", dirname);
            eprintln!("Suggestions:");
            eprintln!("  - Choose a different directory name");
            eprintln!("  - Delete the existing repository on GitHub");
            eprintln!("  - Manually link with: gh repo create <new-name> --source=.");
            return Err(anyhow!("Repository name conflict"));
        }

        eprintln!(
            "Warning: Failed to create GitHub repository: {}",
            stderr.trim()
        );
    }

    Ok(())
}

pub fn create_github_remote_if_possible(path: &Path) -> Result<()> {
    if !is_interactive() {
        return Ok(());
    }

    if !is_gh_installed() {
        eprintln!("Warning: gh CLI not found. Install from https://cli.github.com");
        eprintln!("         to enable GitHub integration.");
        return Ok(());
    }

    if !is_gh_authenticated() {
        eprintln!("Warning: gh CLI not authenticated.");
        eprintln!("         Run 'gh auth login' to enable GitHub integration.");
        return Ok(());
    }

    if has_remote_origin(path) {
        eprintln!("Warning: Remote origin already exists. Skipping GitHub remote creation.");
        return Ok(());
    }

    let visibility = prompt_visibility()?;

    create_github_repo(path, visibility)?;

    Ok(())
}
