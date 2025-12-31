use anyhow::Result;
use skim::prelude::*;
use std::path::PathBuf;
use std::sync::Arc;

use crate::projects::Project;

pub struct ProjectItem {
    pub path: PathBuf,
    pub display: String,
    pub searchable: String,
    pub exists: bool,
}

impl SkimItem for ProjectItem {
    fn text(&self) -> Cow<'_, str> {
        Cow::Borrowed(&self.searchable)
    }

    fn display<'a>(&'a self, _context: DisplayContext<'a>) -> AnsiString<'a> {
        AnsiString::parse(&self.display)
    }
}

pub enum SelectionResult {
    Selected(PathBuf),
    MissingSelected(PathBuf),
    MultiSelected(Vec<PathBuf>),
    Cancelled,
}

/// Check if filter should be case-sensitive (smart-case: sensitive if contains uppercase)
fn is_case_sensitive(filter: &str) -> bool {
    filter.chars().any(|c| c.is_uppercase())
}

/// Apply fuzzy matching using skim's algorithm
pub fn fuzzy_match(text: &str, pattern: &str) -> bool {
    if pattern.is_empty() {
        return true;
    }

    let case_sensitive = is_case_sensitive(pattern);
    let (text, pattern) = if case_sensitive {
        (text.to_string(), pattern.to_string())
    } else {
        (text.to_lowercase(), pattern.to_lowercase())
    };

    // Simple fuzzy match: all pattern chars must appear in order
    let mut pattern_chars = pattern.chars().peekable();
    for text_char in text.chars() {
        if let Some(&pattern_char) = pattern_chars.peek() {
            if text_char == pattern_char {
                pattern_chars.next();
            }
        }
    }
    pattern_chars.peek().is_none()
}

/// Filter projects by multiple terms (AND logic)
pub fn filter_projects<'a>(projects: &[&'a Project], filters: &[String]) -> Vec<&'a Project> {
    if filters.is_empty() {
        return projects.to_vec();
    }

    projects
        .iter()
        .filter(|project| {
            // Build searchable text: path + tags
            let path_str = project.path.display().to_string();
            let tags_str = project.tags.join(" ");
            let searchable = format!("{} {}", path_str, tags_str);

            // All filter terms must match (AND logic)
            filters.iter().all(|filter| fuzzy_match(&searchable, filter))
        })
        .copied()
        .collect()
}

fn create_project_items(projects: &[&Project]) -> Vec<ProjectItem> {
    projects
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

            // Searchable text includes path and tags for matching
            let searchable = format!("{} {}", p.path.display(), p.tags.join(" "));

            ProjectItem {
                path: p.path.clone(),
                display,
                searchable,
                exists,
            }
        })
        .collect()
}

pub fn select_project(
    projects: &[&Project],
    query: Option<&str>,
) -> Result<SelectionResult> {
    let items = create_project_items(projects);

    let items: Vec<Arc<dyn SkimItem>> = items
        .into_iter()
        .map(|item| Arc::new(item) as Arc<dyn SkimItem>)
        .collect();

    let mut options_builder = SkimOptionsBuilder::default();
    options_builder.height(Some("50%")).multi(false);

    if let Some(q) = query {
        options_builder.query(Some(q));
    }

    let options = options_builder.build().unwrap();

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

pub fn select_projects_multi(projects: &[&Project]) -> Result<SelectionResult> {
    let items = create_project_items(projects);

    let items: Vec<Arc<dyn SkimItem>> = items
        .into_iter()
        .map(|item| Arc::new(item) as Arc<dyn SkimItem>)
        .collect();

    let options = SkimOptionsBuilder::default()
        .height(Some("50%"))
        .multi(true)
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
            let paths: Vec<PathBuf> = out
                .selected_items
                .iter()
                .map(|selected| {
                    (**selected)
                        .as_any()
                        .downcast_ref::<ProjectItem>()
                        .unwrap()
                        .path
                        .clone()
                })
                .collect();
            Ok(SelectionResult::MultiSelected(paths))
        }
        _ => Ok(SelectionResult::Cancelled),
    }
}
