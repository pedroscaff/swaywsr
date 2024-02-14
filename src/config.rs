use serde::Deserialize;
use std::collections::HashMap as Map;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

lazy_static::lazy_static! {
    pub static ref EMPTY_MAP: Map<String, String> = Map::new();
    pub static ref EMPTY_OPT_MAP: Map<String, bool> = Map::new();
}

#[derive(Deserialize)]
#[serde(default)]
pub struct Config {
    pub icons: Map<String, char>,
    pub aliases: Map<String, String>,
    pub general: Map<String, String>,
    pub options: Map<String, bool>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            icons: super::icons::NONE.clone(),
            aliases: EMPTY_MAP.clone(),
            general: EMPTY_MAP.clone(),
            options: EMPTY_OPT_MAP.clone(),
        }
    }
}

pub fn read_toml_config(path: &Path) -> anyhow::Result<Config> {
    let mut file = File::open(path)?;
    let mut buffer = String::new();
    file.read_to_string(&mut buffer)?;
    let config: Config = toml::from_str(&buffer)?;
    Ok(config)
}

pub fn xdg_config_home() -> PathBuf {
    // In the unlikely event that $HOME is not set, it doesn't really matter
    // what we fall back on, so use /.config.
    PathBuf::from(std::env::var("XDG_CONFIG_HOME").unwrap_or(format!(
        "{}/.config",
        std::env::var("HOME").unwrap_or_default()
    )))
}
