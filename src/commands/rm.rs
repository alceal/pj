use anyhow::{bail, Result};

use crate::projects::ProjectStore;
use crate::tui::{select_projects_multi, SelectionResult};

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

    match select_projects_multi(&projects)? {
        SelectionResult::MultiSelected(paths) => {
            let mut removed_count = 0;
            for path in &paths {
                if store.remove(path) {
                    eprintln!("Removed: {}", path.display());
                    removed_count += 1;
                }
            }
            if removed_count > 0 {
                store.save()?;
            }
        }
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
