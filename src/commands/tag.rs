use anyhow::{bail, Result};
use dialoguer::{Completion, Input};
use std::path::PathBuf;

use crate::projects::ProjectStore;
use crate::tui::{select_project, SelectionResult};

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

pub fn run(path: Option<PathBuf>, tags: Vec<String>, remove: bool) -> Result<()> {
    let mut store = ProjectStore::load()?;

    let target_path = match path {
        Some(p) => {
            let canonical = p.canonicalize().unwrap_or(p);
            canonical
        }
        None => {
            let projects = store.sorted_by_frecency();
            if projects.is_empty() {
                bail!("No projects tracked. Add a project with: pj -a");
            }

            match select_project(&projects, None)? {
                SelectionResult::Selected(p) => p,
                SelectionResult::MissingSelected(p) => {
                    eprintln!("Project path does not exist: {}", p.display());
                    bail!("Cannot manage tags for missing project");
                }
                SelectionResult::Cancelled => {
                    std::process::exit(130);
                }
            }
        }
    };

    let project = store
        .find_by_path_mut(&target_path)
        .ok_or_else(|| anyhow::anyhow!("Project not found: {}", target_path.display()))?;

    if remove {
        if tags.is_empty() {
            bail!("No tags specified to remove");
        }
        project.remove_tags(&tags);
        eprintln!("Removed tags: {}", tags.join(", "));
    } else if tags.is_empty() {
        let all_tags = store.all_tags();
        let completion = TagCompletion { tags: all_tags };

        let input: String = Input::new()
            .with_prompt("Enter tags (comma-separated)")
            .completion_with(&completion)
            .interact_text()?;

        let new_tags: Vec<String> = input
            .split(',')
            .map(|s| s.trim().to_lowercase())
            .filter(|s| !s.is_empty())
            .collect();

        if new_tags.is_empty() {
            bail!("No tags entered");
        }

        let project = store.find_by_path_mut(&target_path).unwrap();
        project.add_tags(new_tags.clone());
        eprintln!("Added tags: {}", new_tags.join(", "));
    } else {
        project.add_tags(tags.clone());
        eprintln!("Added tags: {}", tags.join(", "));
    }

    let project = store.find_by_path(&target_path).unwrap();
    eprintln!("Current tags: {}", project.tags.join(", "));

    store.save()?;
    Ok(())
}
