use std::path::Path;

use anyhow::Result;
use chrono::{DateTime, Local, Utc};
use crossterm::terminal;
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
    #[tabled(rename = "STATUS")]
    status: String,
}

fn shorten_path(path: &Path, max_width: usize) -> String {
    let full = if let Some(home) = dirs::home_dir() {
        if let Ok(stripped) = path.strip_prefix(&home) {
            format!("~/{}", stripped.display())
        } else {
            path.display().to_string()
        }
    } else {
        path.display().to_string()
    };

    if full.len() <= max_width {
        return full;
    }

    if let Some(rest) = full.strip_prefix("~/") {
        let components: Vec<&str> = rest.split('/').collect();
        if components.len() >= 2 {
            for n in (1..components.len()).rev() {
                let tail = components[components.len() - n..].join("/");
                let shortened = format!("~/.../{}",  tail);
                if shortened.len() <= max_width {
                    return shortened;
                }
            }
            return format!("~/.../{}",  components.last().unwrap());
        }
    }

    full
}

pub fn run() -> Result<()> {
    let store = ProjectStore::load()?;

    let projects = store.sorted_by_frecency();

    if projects.is_empty() {
        eprintln!("No projects found");
        return Ok(());
    }

    let tags_width = projects
        .iter()
        .map(|p| p.tags.join(", ").len())
        .max()
        .unwrap_or(0)
        .max(4);

    let access_width = projects
        .iter()
        .map(|p| p.access_count.to_string().len())
        .max()
        .unwrap_or(0)
        .max(6);

    // 6 borders + 5 columns * 2 padding + fixed columns (LAST ACCESSED=16, STATUS=7)
    let overhead = 16 + tags_width + access_width + 16 + 7;

    let max_path_width = terminal::size()
        .map(|(w, _)| (w as usize).saturating_sub(overhead))
        .unwrap_or(usize::MAX);

    let rows: Vec<ProjectRow> = projects
        .iter()
        .map(|p| {
            let dt = DateTime::<Utc>::from_timestamp(p.last_accessed, 0)
                .map(|dt| dt.with_timezone(&Local).format("%Y-%m-%d %H:%M").to_string())
                .unwrap_or_else(|| "Unknown".to_string());

            ProjectRow {
                path: shorten_path(&p.path, max_path_width),
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

    let table = Table::new(rows);
    println!("{}", table);

    Ok(())
}
