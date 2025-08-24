use pluckrs::utils;
use semver::Version;
use std::{
    process::{Command, Stdio},
    str::{from_utf8, FromStr},
};
mod tmux_utils;

// This file serves as the main entry point to the plugin. All it does it launch
// `tmux popup -E <fzf-handler>`. This could probably have been done in bash, but feels nice
// to keep everything in Rust.

fn get_tmux_version() -> Result<Version, String> {
    // 'tmux set-env -g TMUX_VERSION $(tmux -V | sed "s/^tmux \([0-9\.]*\).*/\1/")'
    let tmux_version_child = Command::new("tmux")
        .arg("-V")
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn 'tmux -V'. Error: {:?}", e))?;

    let tmux_stdout = tmux_version_child
        .stdout
        .ok_or_else(|| "Failed to read stdout from tmux version command")?;

    let sed_child = Command::new("sed")
        .arg("s/^tmux \\([0-9\\.]*\\).*/\\1/")
        .stdin(Stdio::from(tmux_stdout))
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn 'sed' command. Error: {}", e))?;

    let sed_out = sed_child
        .wait_with_output()
        .map_err(|e| format!("Failed to read output from sed. Error: {}", e))?;
    let sed_stdout_utf8 = from_utf8(&sed_out.stdout)
        .map_err(|e| format!("Failed to convert sed stdout to utf8. Error: {}", e))?;

    // This is going to give us something like 2.x or 3.x, i.e. something without the
    // required patch versioning.
    let sed_stdout_utf8_trimmed = sed_stdout_utf8.trim();

    // Adding patch versioning to semver
    let normalized_version_string = sed_stdout_utf8_trimmed.to_owned() + ".0";

    let version = Version::parse(&normalized_version_string).map_err(|e| {
        format!(
            "Failed to parse semantic version from sed output. Error: {}",
            e
        )
    })?;

    Ok(version)
}

fn main() -> Result<(), String> {
    let configuration = utils::get_home_config_file()?;

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

    let tmux_version = get_tmux_version()?;
    let popup_cutoff_version = Version::from_str("3.2.0")
        .map_err(|e| format!("Failed to create tmux popup cutoff version. Error: {}", e))?;
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
            .output()
            .map_err(|e| e.to_string())?;
    }
    else {
        // TODO: For some reason, this gives no results...
        Command::new("tmux")
            .arg("split-window") 
            .arg("env")
            .arg(format!("TMUX_PANE={}", tmux_pane))
            .arg(fzf_handler_path)
            .output()
            .map_err(|e| e.to_string())?;
    }


    Ok(())
}
