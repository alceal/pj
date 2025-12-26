use anyhow::{bail, Result};
use std::env;

use crate::projects::ProjectStore;
use crate::tui::{select_project_for_removal, SelectionResult};

pub fn run(missing: bool) -> Result<()> {
    let mut store = ProjectStore::load()?;

    if missing {
        let removed = store.remove_missing();
        if removed > 0 {
            eprintln!("Removed {} projects with non-existent paths", removed);
            store.save()?;
        } else {
            eprintln!("No missing projects found");
        }
        return Ok(());
    }

    let projects = store.sorted_by_frecency();
    if projects.is_empty() {
        bail!("No projects tracked. Add a project with: pj -a");
    }

    let current_dir = env::current_dir().ok().and_then(|p| p.canonicalize().ok());

    match select_project_for_removal(&projects, current_dir.as_ref())? {
        SelectionResult::Selected(path) | SelectionResult::MissingSelected(path) => {
            if store.remove(&path) {
                eprintln!("Removed: {}", path.display());
                store.save()?;
            } else {
                eprintln!("Project not found: {}", path.display());
            }
        }
        SelectionResult::Cancelled => {
            std::process::exit(130);
        }
    }

    Ok(())
}
