use crate::config;
use dirs::home_dir;
use itertools::Itertools;
use regex::Regex;
use std::{
    io::Write,
    path::PathBuf,
    process::{Command, Output, Stdio},
};

pub mod colors {
    pub const RED: &str = "\x1b[0;31m";
    pub const GREEN: &str = "\x1b[0;32m";
    pub const BLUE: &str = "\x1b[0;34m";
    pub const PURPLE: &str = "\x1b[0;35m";
    pub const CYAN: &str = "\x1b[0;36m";
    pub const WHITE: &str = "\x1b[0;37m";
    pub const YELLOW: &str = "\x1b[0;33m";
    pub const OFF: &str = "\x1b[0m";
    pub const BOLD: &str = "\x1b[1m";
}

fn config_path_from_home_dir(home_dir: &PathBuf) -> PathBuf {
    home_dir.join(".config").join("pluckrs").join("config.toml")
}

pub fn get_home_config_file() -> Result<config::Config, String> {
    let home_directory = home_dir().ok_or_else(|| "Unable to find home directory!".to_string())?;
    let config_file_path = config_path_from_home_dir(&home_directory);

    let configuration = config::read_config(&config_file_path).map_err(|e| {
        format!(
            "Failed to read configuration! Make sure you have your configuration at \
            `{}`. Error was: '{}'",
            config_file_path.display(),
            e.to_string()
        )
    })?;
    return Ok(configuration);
}

pub fn copy_into_clipboard(text: &str, clip_tool: &str) -> Result<(), String> {
    let mut copy_cmd = Command::new(clip_tool);
    copy_cmd.stdin(Stdio::piped());

    let mut process_handle = copy_cmd.spawn().map_err(|e| e.to_string())?;
    let mut stdin_handle = process_handle
        .stdin
        .take()
        .ok_or("Unable to get stdin handle from copy process")?;

    stdin_handle.write_all(text.as_bytes()).map_err(|e| {
        format!(
            "Failed to write text to copy process stdin with error {}",
            e
        )
    })?;
    drop(stdin_handle);
    process_handle.wait().map_err(|e| e.to_string())?;

    Ok(())
}

pub fn insert_text(text: &str, tmux_pane: &str) -> Result<(), std::io::Error> {
    Command::new("tmux")
        .arg("send-keys")
        .arg("-t")
        .arg(tmux_pane)
        .arg(text)
        .output()?;

    Ok(())
}

pub fn get_filtered_data_from_lines(
    lines: &Vec<&str>,
    regex_pattern: &Regex,
    min_length: u8,
) -> String {
    // Finds all matches in all lines and joins the results into a single string
    // Only outputs unique matches
    let result = lines
        .iter()
        .flat_map(|line| {
            regex_pattern
                .find_iter(line)
                .map(|m| m.as_str())
                .collect::<Vec<&str>>()
        })
        .filter(|l| l.len() >= usize::from(min_length))
        .unique()
        .map(|l| l.trim())
        .join("\n");
    return result;
}

pub fn launch_fzf(
    query: &str,
    mode: &str,
    buffer_contents: &str,
    filter_button: &str,
    copy_button: &str,
    insert_button: &str,
) -> Result<Output, String> {
    let mut fzf_cmd = Command::new("fzf");

    fzf_cmd.arg(format!(
        "--expect={},{},{}",
        filter_button, copy_button, insert_button
    ));
    fzf_cmd.arg("--expect=ctrl-g,esc"); // Adding basic buttons to ensure always capturing a key
    fzf_cmd.arg("--print-query"); // Get the query when fzf finishes
    fzf_cmd.arg(format!("--query={}", query)); // Inject the previous query

    let header = format!(
        "{}PLUCKRS{} | mode: {}[{}]{} | Toggle filter: {} | Copy: {} | Insert: {}",
        colors::GREEN,
        colors::OFF,
        colors::GREEN,
        mode,
        colors::OFF,
        filter_button,
        copy_button,
        insert_button
    );
    fzf_cmd.arg(format!("--header={}", header));

    fzf_cmd.stdin(Stdio::piped());
    fzf_cmd.stdout(Stdio::piped());

    let mut fzf_process_handle = fzf_cmd.spawn().map_err(|e| e.to_string())?;
    let mut fzf_stdin_handle = fzf_process_handle
        .stdin
        .take()
        .ok_or("Unable to get stdin handle from fzf process")?;

    fzf_stdin_handle
        .write_all(buffer_contents.as_bytes())
        .map_err(|e| format!("Failed to write buffer to fzf stdin with error {}", e))?;

    drop(fzf_stdin_handle);
    let output = fzf_process_handle
        .wait_with_output()
        .map_err(|e| e.to_string())?;

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_path_is_built_correctly() {
        let dummy_home_dir = PathBuf::from("/Users/my_home");
        let built_config_path = config_path_from_home_dir(&dummy_home_dir);
        assert_eq!(
            built_config_path,
            PathBuf::from("/Users/my_home/.config/pluckrs/config.toml")
        );
    }
}
