use serde::Deserialize;
use std::{collections::HashMap, fs, path::PathBuf};

#[derive(Deserialize, Debug)]
pub struct Config {
    pub general: General,
    pub keybinds: Keybinds,
    pub regexes: HashMap<String, String>,
}

#[derive(Deserialize, Debug)]
pub struct General {
    pub backward_history: u32,
    pub clip_tool: Option<String>,
    pub regex_order: Vec<String>,
    pub popup_width: String,
    pub popup_height: String,
}

#[derive(Debug, Deserialize)]
pub struct Keybinds {
    pub insert: String,
    pub copy: String,
    pub filter: String,
}


pub fn read_config(file_path: PathBuf) -> Result<Config, String> {
    let contents = fs::read_to_string(file_path).map_err(|e| e.to_string())?;
    let config: Config = toml::from_str(&contents).map_err(|e| e.to_string())?;

    Ok(config)
}
