use std::process::Command;
mod tmux_utils;

// This file serves as the main entry point to the plugin. All it does it launch
// `tmux popup -E <fzf-handler>`. This could probably have been done in bash, but feels nice
// to keep everything in Rust.

fn main() -> Result<(), String> {
    let mut fzf_handler_path = std::env::current_exe().map_err(|e| {
        format!(
            "Getting path of current executable failed with error: {}",
            e
        )
    })?;
    fzf_handler_path.set_file_name("fzf_handler");

    let _ = Command::new("tmux")
        .arg("popup")
        .arg("-E")
        .arg(fzf_handler_path)
        .output();

    Ok(())
}
