use semver::Version;
use std::{
    process::{Command, Stdio},
    str::from_utf8,
};

pub fn get_tmux_pane_height() -> Result<u16, String> {
    let process = Command::new("tmux")
        .arg("display-message")
        .arg("-p")
        .arg("#{pane_height}")
        .output()
        .map_err(|e| format!("'tmux display-message' failed with error {}.", e))?;

    let pane_height_str = from_utf8(&process.stdout)
        .map_err(|e| {
            format!(
                "Failed converting stdout from utf8 to string. Error was {}",
                e
            )
        })?
        .trim();

    let pane_height = pane_height_str.parse::<u16>().map_err(|e| {
        format!(
            "Failed when parsing string, '{}' to u16. Error was {}",
            pane_height_str, e
        )
    })?;

    Ok(pane_height)
}

pub fn get_tmux_buffer_contents(
    pane_id: &str,
    pane_height: u16,
    backward_history: u32,
) -> Result<String, String> {
    let mut tmux_cmd = Command::new("tmux");
    tmux_cmd
        .arg("capture-pane")
        .arg("-p") // Send result to stdout
        .arg("-S") // Starting line
        .arg(format!("-{}", backward_history))
        .arg("-E") // End line
        .arg(format!("{}", pane_height))
        .arg("-t")
        .arg(format!("{}", pane_id));

    let cmd_output = tmux_cmd
        .output()
        .map_err(|e| format!("'tmux capture-pane' failed with error: {}", e))?;

    let buffer_contents = from_utf8(&cmd_output.stdout).map_err(|e| e.to_string())?;
    return Ok(buffer_contents.to_string());
}

pub fn get_tmux_version() -> Result<Version, String> {
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
