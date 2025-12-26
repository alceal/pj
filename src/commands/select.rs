use anyhow::{bail, Result};
use dialoguer::Confirm;
use std::process::Command;

use crate::config::Config;
use crate::projects::ProjectStore;
use crate::tui::{select_project, SelectionResult};

pub fn run(
    tags: Option<String>,
    editor_override: Option<String>,
    cd_override: Option<bool>,
) -> Result<()> {
    let config = Config::load()?;
    let mut store = ProjectStore::load()?;

    let filter_tags: Vec<String> = tags
        .map(|t| t.split(',').map(|s| s.trim().to_lowercase()).collect())
        .unwrap_or_default();

    let projects = if filter_tags.is_empty() {
        store.sorted_by_frecency()
    } else {
        let mut filtered = store.filter_by_tags(&filter_tags);
        filtered.sort_by(|a, b| b.frecency().partial_cmp(&a.frecency()).unwrap());
        filtered
    };

    if projects.is_empty() {
        bail!("No projects found. Add a project with: pj -a");
    }

    let selected_path = match select_project(&projects, None)? {
        SelectionResult::Selected(path) => path,
        SelectionResult::MissingSelected(path) => {
            eprintln!("Project path does not exist: {}", path.display());
            let remove = Confirm::new()
                .with_prompt("Remove from tracking?")
                .default(true)
                .interact()?;

            if remove {
                store.remove(&path);
                store.save()?;
                eprintln!("Removed: {}", path.display());
            }
            std::process::exit(1);
        }
        SelectionResult::Cancelled => {
            std::process::exit(130);
        }
    };

    if let Some(project) = store.find_by_path_mut(&selected_path) {
        project.update_access();
    }
    store.save()?;

    let editor = editor_override.unwrap_or(config.editor);
    let should_cd = cd_override.unwrap_or(config.cd_on_select);

    Command::new(&editor)
        .arg(&selected_path)
        .spawn()
        .map_err(|e| anyhow::anyhow!("Failed to launch editor '{}': {}", editor, e))?;

    if should_cd {
        println!("{}", selected_path.display());
    }

    Ok(())
}
