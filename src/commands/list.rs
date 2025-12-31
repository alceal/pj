use anyhow::Result;
use chrono::{DateTime, Local, Utc};
use tabled::{Table, Tabled};

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
                path: p.path.display().to_string(),
                tags: p.tags.join(", "),
                access_count: p.access_count,
                last_accessed: dt,
            }
        })
        .collect();

    let table = Table::new(rows).to_string();
    println!("{}", table);

    Ok(())
}
