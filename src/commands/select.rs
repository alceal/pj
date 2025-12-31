use anyhow::{bail, Result};
use dialoguer::Confirm;
use std::process::Command;

use crate::config::Config;
use crate::projects::ProjectStore;
use crate::tui::{filter_projects, select_project, SelectionResult};

pub fn run(
    filters: Vec<String>,
    editor_override: Option<String>,
    cd_override: Option<bool>,
) -> Result<()> {
    let config = Config::load()?;
    let mut store = ProjectStore::load()?;

    let all_projects = store.sorted_by_frecency();

    if all_projects.is_empty() {
        bail!("No projects found. Add a project with: pj -a");
    }

    // Apply filter logic
    let filtered_projects = filter_projects(&all_projects, &filters);

    // Filter out missing projects for auto-selection consideration
    let existing_filtered: Vec<_> = filtered_projects
        .iter()
        .filter(|p| p.exists())
        .copied()
        .collect();

    let selected_path = if filters.is_empty() {
        // No filter: show all projects in skim
        match select_project(&all_projects, None)? {
            SelectionResult::Selected(path) => path,
            SelectionResult::MissingSelected(path) => {
                handle_missing_project(&mut store, &path)?;
                std::process::exit(1);
            }
            SelectionResult::Cancelled | SelectionResult::MultiSelected(_) => {
                std::process::exit(130);
            }
        }
    } else if existing_filtered.len() == 1 {
        // Single match: auto-open silently
        existing_filtered[0].path.clone()
    } else {
        // Zero or multiple matches: show skim with filter pre-populated
        let query = filters.join(" ");
        let projects_to_show = if filtered_projects.is_empty() {
            // Zero matches: show all projects so user can modify query
            &all_projects
        } else {
            // Multiple matches: show filtered projects
            &filtered_projects
        };

        match select_project(projects_to_show, Some(&query))? {
            SelectionResult::Selected(path) => path,
            SelectionResult::MissingSelected(path) => {
                handle_missing_project(&mut store, &path)?;
                std::process::exit(1);
            }
            SelectionResult::Cancelled | SelectionResult::MultiSelected(_) => {
                std::process::exit(130);
            }
        }
    };

    // Update frecency for selected project
    if let Some(project) = store.find_by_path_mut(&selected_path) {
        project.update_access();
    }
    store.save()?;

    // Handle editor
    let should_open_editor = match &editor_override {
        Some(e) if e.is_empty() => false, // --no-editor was used
        _ => true,
    };

    if should_open_editor {
        let editor = editor_override
            .filter(|e| !e.is_empty())
            .unwrap_or(config.editor);

        Command::new(&editor)
            .arg(&selected_path)
            .spawn()
            .map_err(|e| anyhow::anyhow!("Failed to launch editor '{}': {}", editor, e))?;
    }

    // Handle cd output
    let should_cd = cd_override.unwrap_or(config.cd_on_select);
    if should_cd {
        println!("{}", selected_path.display());
    }

    Ok(())
}

fn handle_missing_project(store: &mut ProjectStore, path: &std::path::Path) -> Result<()> {
    eprintln!("Project path does not exist: {}", path.display());
    let remove = Confirm::new()
        .with_prompt("Remove from tracking?")
        .default(true)
        .interact()?;

    if remove {
        store.remove(path);
        store.save()?;
        eprintln!("Removed: {}", path.display());
    }
    Ok(())
}
