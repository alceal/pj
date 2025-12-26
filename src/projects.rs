use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use crate::config::Config;
use crate::frecency::calculate_frecency;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Project {
    pub path: PathBuf,
    pub tags: Vec<String>,
    pub last_accessed: i64,
    pub access_count: u32,
}

impl Project {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            tags: Vec::new(),
            last_accessed: chrono::Utc::now().timestamp(),
            access_count: 0,
        }
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags.into_iter().map(|t| t.to_lowercase()).collect();
        self
    }

    pub fn update_access(&mut self) {
        self.last_accessed = chrono::Utc::now().timestamp();
        self.access_count += 1;
    }

    pub fn add_tags(&mut self, tags: Vec<String>) {
        for tag in tags {
            let tag_lower = tag.to_lowercase();
            if !self.tags.contains(&tag_lower) {
                self.tags.push(tag_lower);
            }
        }
    }

    pub fn remove_tags(&mut self, tags: &[String]) {
        let tags_lower: Vec<String> = tags.iter().map(|t| t.to_lowercase()).collect();
        self.tags.retain(|t| !tags_lower.contains(t));
    }

    pub fn matches_tag_filter(&self, filter_tag: &str) -> bool {
        let filter_lower = filter_tag.to_lowercase();
        self.tags.iter().any(|tag| {
            tag == &filter_lower || tag.starts_with(&format!("{}/", filter_lower))
        })
    }

    pub fn exists(&self) -> bool {
        self.path.exists()
    }

    pub fn frecency(&self) -> f64 {
        calculate_frecency(self.last_accessed, self.access_count)
    }
}

#[derive(Debug, Default)]
pub struct ProjectStore {
    projects: Vec<Project>,
}

impl ProjectStore {
    pub fn load() -> Result<Self> {
        let path = Config::projects_path()?;
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read projects file: {}", path.display()))?;
        let projects: Vec<Project> =
            serde_json::from_str(&content).with_context(|| "Failed to parse projects file")?;
        Ok(Self { projects })
    }

    pub fn save(&self) -> Result<()> {
        let pj_dir = Config::pj_dir()?;
        if !pj_dir.exists() {
            fs::create_dir_all(&pj_dir)
                .with_context(|| format!("Failed to create directory: {}", pj_dir.display()))?;
        }
        let path = Config::projects_path()?;
        let content =
            serde_json::to_string_pretty(&self.projects).context("Failed to serialize projects")?;
        fs::write(&path, content)
            .with_context(|| format!("Failed to write projects file: {}", path.display()))?;
        Ok(())
    }

    pub fn add(&mut self, project: Project) -> bool {
        if let Some(existing) = self.find_by_path_mut(&project.path) {
            existing.last_accessed = chrono::Utc::now().timestamp();
            existing.add_tags(project.tags);
            false
        } else {
            self.projects.push(project);
            true
        }
    }

    pub fn find_by_path(&self, path: &Path) -> Option<&Project> {
        self.projects.iter().find(|p| p.path == path)
    }

    pub fn find_by_path_mut(&mut self, path: &Path) -> Option<&mut Project> {
        self.projects.iter_mut().find(|p| p.path == path)
    }

    pub fn remove(&mut self, path: &Path) -> bool {
        let len_before = self.projects.len();
        self.projects.retain(|p| p.path != path);
        self.projects.len() < len_before
    }

    pub fn remove_missing(&mut self) -> usize {
        let len_before = self.projects.len();
        self.projects.retain(|p| p.path.exists());
        len_before - self.projects.len()
    }

    pub fn filter_by_tags(&self, tags: &[String]) -> Vec<&Project> {
        if tags.is_empty() {
            return self.projects.iter().collect();
        }
        self.projects
            .iter()
            .filter(|p| tags.iter().any(|t| p.matches_tag_filter(t)))
            .collect()
    }

    pub fn sorted_by_frecency(&self) -> Vec<&Project> {
        let mut projects: Vec<&Project> = self.projects.iter().collect();
        projects.sort_by(|a, b| b.frecency().partial_cmp(&a.frecency()).unwrap());
        projects
    }

    pub fn all_tags(&self) -> Vec<String> {
        let mut tags: Vec<String> = self
            .projects
            .iter()
            .flat_map(|p| p.tags.clone())
            .collect();
        tags.sort();
        tags.dedup();
        tags
    }
}
