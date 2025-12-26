use anyhow::{Context, Result};
use std::env;
use std::path::PathBuf;
use std::process::Command;

use crate::config::Config;
use crate::github::create_github_remote_if_possible;
use crate::projects::{Project, ProjectStore};

fn is_git_repo(path: &PathBuf) -> bool {
    path.join(".git").exists()
}

fn git_init(path: &PathBuf) -> Result<bool> {
    let output = Command::new("git")
        .arg("init")
        .current_dir(path)
        .output();

    match output {
        Ok(out) if out.status.success() => Ok(true),
        Ok(_) => Ok(false),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            eprintln!("Warning: git is not installed, skipping git init");
            Ok(false)
        }
        Err(e) => Err(e).context("Failed to run git init"),
    }
}

pub fn run(tags: Option<String>) -> Result<()> {
    let config = Config::load()?;
    let mut store = ProjectStore::load()?;

    let cwd = env::current_dir().context("Failed to get current directory")?;
    let canonical_path = cwd
        .canonicalize()
        .context("Failed to resolve canonical path")?;

    let tags_vec: Vec<String> = tags
        .map(|t| t.split(',').map(|s| s.trim().to_lowercase()).collect())
        .unwrap_or_default();

    let project = Project::new(canonical_path.clone()).with_tags(tags_vec.clone());

    let is_new = store.add(project);

    if is_new {
        eprintln!("Added: {}", canonical_path.display());

        if config.git_init_on_add && !is_git_repo(&canonical_path) {
            if git_init(&canonical_path)? {
                eprintln!("Initialized git repository");
            }
        }

        if config.gh_create_on_add {
            create_github_remote_if_possible(&canonical_path)?;
        }
    } else {
        eprintln!(
            "Already tracked: {} (updated timestamp)",
            canonical_path.display()
        );
    }

    if !tags_vec.is_empty() {
        eprintln!("Tags: {}", tags_vec.join(", "));
    }

    store.save()?;
    Ok(())
}
