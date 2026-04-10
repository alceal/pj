use std::path::Path;

use anyhow::Result;
use chrono::{DateTime, Local, Utc};
use crossterm::terminal;
use tabled::{
    settings::{peaker::PriorityMax, Width},
    Table, Tabled,
};

use crate::projects::ProjectStore;

#[derive(Tabled)]
struct ProjectRow {
    #[tabled(rename = "PATH")]
    path: String,
    #[tabled(rename = "TAGS")]
    tags: String,
    #[tabled(rename = "ACCESS")]
    access_count: u32,
    #[tabled(rename = "LAST ACCESSED")]
    last_accessed: String,
    #[tabled(rename = "STATUS")]
    status: String,
}

fn shorten_path(path: &Path) -> String {
    if let Some(home) = dirs::home_dir() {
        if let Ok(stripped) = path.strip_prefix(&home) {
            return format!("~/{}", stripped.display());
        }
    }
    path.display().to_string()
}

pub fn run() -> Result<()> {
    let store = ProjectStore::load()?;

    let projects = store.sorted_by_frecency();

    if projects.is_empty() {
        eprintln!("No projects found");
        return Ok(());
    }

    let rows: Vec<ProjectRow> = projects
        .iter()
        .map(|p| {
            let dt = DateTime::<Utc>::from_timestamp(p.last_accessed, 0)
                .map(|dt| dt.with_timezone(&Local).format("%Y-%m-%d %H:%M").to_string())
                .unwrap_or_else(|| "Unknown".to_string());

            ProjectRow {
                path: shorten_path(&p.path),
                tags: p.tags.join(", "),
                access_count: p.access_count,
                last_accessed: dt,
                status: if p.exists() {
                    "OK".to_string()
                } else {
                    "MISSING".to_string()
                },
            }
        })
        .collect();

    let mut table = Table::new(rows);

    if let Ok((width, _)) = terminal::size() {
        table.with(
            Width::truncate(width as usize)
                .suffix("..")
                .priority(PriorityMax::right()),
        );
    }

    println!("{}", table);

    Ok(())
}
