use pluckrs::tmux_utils;
use pluckrs::utils;
use semver::Version;
use std::process::Command;
use std::{error::Error, str::FromStr};

// This file serves as the main entry point to the plugin. All it does it launch
// `tmux popup -E <fzf-handler>`. This could probably have been done in bash, but feels nice
// to keep everything in Rust.

fn main() -> Result<(), Box<dyn Error>> {
    let configuration = utils::get_home_config_file()?;

    let height = configuration.general.popup_height;
    let width = configuration.general.popup_width;

    let mut fzf_handler_path = std::env::current_exe()?;
    fzf_handler_path.set_file_name("fzf_handler");

    // Needed to allow the subprocess to know which pane to use
    let tmux_pane = std::env::var("TMUX_PANE")?;

    let tmux_version = tmux_utils::get_tmux_version()?;
    let popup_cutoff_version = Version::from_str("3.2.0").expect("Hardcoded semver should work!");

    if tmux_version >= popup_cutoff_version {
        Command::new("tmux")
            .arg("popup")
            .arg("-h")
            .arg(height)
            .arg("-w")
            .arg(width)
            .arg("env")
            .arg(format!("TMUX_PANE={}", tmux_pane))
            .arg(fzf_handler_path)
            .output()?;
    } else {
        Command::new("tmux")
            .arg("split-window")
            .arg("env")
            .arg(format!("TMUX_PANE={}", tmux_pane))
            .arg(fzf_handler_path)
            .output()?;
    }

    Ok(())
}
