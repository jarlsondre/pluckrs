use serde::Deserialize;
use std::{collections::HashMap, env::consts::OS, fs, path::PathBuf};

fn default_clipboard_tool() -> String {
    if OS == "macos" {
        return "pbcopy".to_string();
    } else if OS == "linux" {
        if std::env::var("XDG_SESSION_TYPE").as_deref() == Ok("wayland") {
            return "wl-copy".to_string();
        } else {
            return "xclip -i -selection clipboard >/dev/null".to_string();
        }
    } else {
        panic!("Operating system is not supported! Only 'macos' and 'linux' are supported.")
    }
}

fn default_backward_history() -> u32 {
    10000
}
fn default_min_length() -> u8 {
    10
}
fn default_popup_width() -> String {
    String::from("60%")
}
fn default_popup_height() -> String {
    String::from("90%")
}

fn default_insert_key() -> String {
    String::from("tab")
}
fn default_copy_key() -> String {
    String::from("enter")
}
fn default_filter_key() -> String {
    String::from("ctrl-f")
}

#[derive(Deserialize, Debug)]
pub struct Config {
    pub general: General,
    pub keybinds: Keybinds,
    pub regexes: HashMap<String, String>,
}

#[derive(Deserialize, Debug)]
pub struct General {
    #[serde(default = "default_backward_history")]
    pub backward_history: u32,

    #[serde(default = "default_min_length")]
    pub min_length: u8,
    #[serde(default = "default_clipboard_tool")]
    pub clip_tool: String,
    pub regex_order: Vec<String>,

    #[serde(default = "default_popup_width")]
    pub popup_width: String,

    #[serde(default = "default_popup_height")]
    pub popup_height: String,
}

#[derive(Debug, Deserialize)]
pub struct Keybinds {
    #[serde(default = "default_insert_key")]
    pub insert: String,

    #[serde(default = "default_copy_key")]
    pub copy: String,

    #[serde(default = "default_filter_key")]
    pub filter: String,
}

pub fn read_config(file_path: &PathBuf) -> Result<Config, String> {
    let contents = fs::read_to_string(file_path).map_err(|e| e.to_string())?;
    let config: Config = toml::from_str(&contents).map_err(|e| e.to_string())?;

    Ok(config)
}
