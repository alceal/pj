use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub editor: String,
    pub cd_on_select: bool,
    pub git_init_on_add: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            editor: "code".to_string(),
            cd_on_select: true,
            git_init_on_add: true,
        }
    }
}

impl Config {
    pub fn pj_dir() -> Result<PathBuf> {
        let home = dirs::home_dir().context("Could not find home directory")?;
        Ok(home.join(".pj"))
    }

    pub fn config_path() -> Result<PathBuf> {
        Ok(Self::pj_dir()?.join("config.toml"))
    }

    pub fn projects_path() -> Result<PathBuf> {
        Ok(Self::pj_dir()?.join("projects.json"))
    }

    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;
        let config: Config =
            toml::from_str(&content).with_context(|| "Failed to parse config file")?;
        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let pj_dir = Self::pj_dir()?;
        if !pj_dir.exists() {
            fs::create_dir_all(&pj_dir)
                .with_context(|| format!("Failed to create directory: {}", pj_dir.display()))?;
        }
        let path = Self::config_path()?;
        let content = toml::to_string_pretty(self).context("Failed to serialize config")?;
        fs::write(&path, content)
            .with_context(|| format!("Failed to write config file: {}", path.display()))?;
        Ok(())
    }

}
