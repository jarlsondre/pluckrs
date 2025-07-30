use dirs::home_dir;
use pluckrs::config;
use std::process::Command;
mod tmux_utils;

// This file serves as the main entry point to the plugin. All it does it launch
// `tmux popup -E <fzf-handler>`. This could probably have been done in bash, but feels nice
// to keep everything in Rust.

fn main() -> Result<(), String> {
    let home_directory = match home_dir() {
        Some(val) => val,
        None => {
            return Err("Unable to find home directory!".to_string());
        }
    };
    let config_file_path = home_directory
        .join(".config")
        .join("pluckrs")
        .join("config.toml");

    let configuration = config::read_config(config_file_path).map_err(|e| {
        format!(
            "Failed to read configuration! Make sure you have your configuration at \
            `~/.config/pluckrs/config.toml`. Error was: '{}'",
            e.to_string()
        )
    })?;

    let height = configuration.general.popup_height;
    let width = configuration.general.popup_width;

    let mut fzf_handler_path = std::env::current_exe().map_err(|e| {
        format!(
            "Getting path of current executable failed with error: {}",
            e
        )
    })?;
    fzf_handler_path.set_file_name("fzf_handler");

    // Allowing the subprocess to see the active pane
    let tmux_pane = std::env::var("TMUX_PANE").map_err(|e| e.to_string())?;

    Command::new("tmux")
        .arg("popup")
        .arg("-e")
        .arg(format!("TMUX_PANE={}", tmux_pane))
        .arg("-h")
        .arg(height)
        .arg("-w")
        .arg(width)
        // The -E has to be the last argument for this to work
        .arg("-E")
        .arg(fzf_handler_path)
        .output()
        .map_err(|e| e.to_string())?;

    Ok(())
}
