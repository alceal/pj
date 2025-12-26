use anyhow::Result;
use skim::prelude::*;
use std::path::PathBuf;
use std::sync::Arc;

use crate::projects::Project;

pub struct ProjectItem {
    pub path: PathBuf,
    pub display: String,
    pub exists: bool,
}

impl SkimItem for ProjectItem {
    fn text(&self) -> Cow<'_, str> {
        Cow::Borrowed(&self.display)
    }
}

pub enum SelectionResult {
    Selected(PathBuf),
    MissingSelected(PathBuf),
    Cancelled,
}

pub fn select_project(projects: &[&Project], default_path: Option<&PathBuf>) -> Result<SelectionResult> {
    let mut items: Vec<ProjectItem> = projects
        .iter()
        .map(|p| {
            let exists = p.exists();
            let tags_str = if p.tags.is_empty() {
                String::new()
            } else {
                format!(" [{}]", p.tags.join(", "))
            };
            let missing_indicator = if exists { "" } else { " [MISSING]" };
            let display = format!("{}{}{}", p.path.display(), tags_str, missing_indicator);
            ProjectItem {
                path: p.path.clone(),
                display,
                exists,
            }
        })
        .collect();

    if let Some(default) = default_path {
        if let Some(pos) = items.iter().position(|item| &item.path == default) {
            let item = items.remove(pos);
            items.insert(0, item);
        }
    }

    let items: Vec<Arc<dyn SkimItem>> = items
        .into_iter()
        .map(|item| Arc::new(item) as Arc<dyn SkimItem>)
        .collect();

    let options = SkimOptionsBuilder::default()
        .height(Some("50%"))
        .multi(false)
        .build()
        .unwrap();

    let (tx, rx): (SkimItemSender, SkimItemReceiver) = unbounded();
    for item in items {
        let _ = tx.send(item);
    }
    drop(tx);

    let output = Skim::run_with(&options, Some(rx));

    match output {
        Some(out) if out.is_abort => Ok(SelectionResult::Cancelled),
        Some(out) if !out.selected_items.is_empty() => {
            let selected = &out.selected_items[0];
            let item = (**selected)
                .as_any()
                .downcast_ref::<ProjectItem>()
                .unwrap();
            if item.exists {
                Ok(SelectionResult::Selected(item.path.clone()))
            } else {
                Ok(SelectionResult::MissingSelected(item.path.clone()))
            }
        }
        _ => Ok(SelectionResult::Cancelled),
    }
}

pub fn select_project_for_removal(projects: &[&Project], current_dir: Option<&PathBuf>) -> Result<SelectionResult> {
    select_project(projects, current_dir)
}
