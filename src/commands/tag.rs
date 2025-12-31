use anyhow::{bail, Result};
use dialoguer::{Completion, Input};
use std::env;
use std::path::PathBuf;

use crate::projects::ProjectStore;
use crate::tui::{select_projects_multi, SelectionResult};

struct TagCompletion {
    tags: Vec<String>,
}

impl Completion for TagCompletion {
    fn get(&self, input: &str) -> Option<String> {
        let input_lower = input.to_lowercase();
        self.tags
            .iter()
            .find(|tag| tag.starts_with(&input_lower))
            .cloned()
    }
}

/// Parse comma-separated tags from a string
fn parse_tags(tags_str: &str) -> Vec<String> {
    tags_str
        .split(',')
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Prompt user for tags with completion
fn prompt_for_tags(all_tags: Vec<String>) -> Result<Vec<String>> {
    let completion = TagCompletion { tags: all_tags };

    let input: String = Input::new()
        .with_prompt("Enter tags (comma-separated)")
        .completion_with(&completion)
        .interact_text()?;

    let tags = parse_tags(&input);
    if tags.is_empty() {
        bail!("No tags entered");
    }
    Ok(tags)
}

pub fn run(tags: Option<String>, path: Option<PathBuf>, remove: bool) -> Result<()> {
    let mut store = ProjectStore::load()?;

    // Parse tags if provided
    let tags_vec: Vec<String> = tags.as_ref().map(|t| parse_tags(t)).unwrap_or_default();

    // Determine target path(s)
    let target_paths: Vec<PathBuf> = if let Some(p) = path {
        // Path provided: resolve it (handle "." for current directory)
        let resolved = if p.as_os_str() == "." {
            env::current_dir()?
        } else {
            p.canonicalize().unwrap_or(p)
        };
        vec![resolved]
    } else {
        // No path: use multi-select
        let projects = store.sorted_by_frecency();
        if projects.is_empty() {
            bail!("No projects tracked. Add a project with: pj -a");
        }

        match select_projects_multi(&projects)? {
            SelectionResult::MultiSelected(paths) => paths,
            SelectionResult::Selected(path) => vec![path],
            SelectionResult::MissingSelected(path) => {
                eprintln!("Project path does not exist: {}", path.display());
                bail!("Cannot manage tags for missing project");
            }
            SelectionResult::Cancelled => {
                std::process::exit(130);
            }
        }
    };

    if target_paths.is_empty() {
        bail!("No projects selected");
    }

    // Determine tags to apply
    let final_tags = if tags_vec.is_empty() {
        // No tags provided: prompt for them
        let all_tags = store.all_tags();
        prompt_for_tags(all_tags)?
    } else {
        tags_vec
    };

    // Apply tags to all selected projects
    for target_path in &target_paths {
        let project = store
            .find_by_path_mut(target_path)
            .ok_or_else(|| anyhow::anyhow!("Project not found: {}", target_path.display()))?;

        if remove {
            project.remove_tags(&final_tags);
        } else {
            project.add_tags(final_tags.clone());
        }
    }

    // Report results
    let action = if remove { "Removed" } else { "Added" };
    eprintln!("{} tags: {}", action, final_tags.join(", "));

    if target_paths.len() == 1 {
        let project = store.find_by_path(&target_paths[0]).unwrap();
        eprintln!("Current tags: {}", project.tags.join(", "));
    } else {
        eprintln!("Updated {} projects", target_paths.len());
    }

    store.save()?;
    Ok(())
}
