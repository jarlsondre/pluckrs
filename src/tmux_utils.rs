use std::process::Command;
use std::str::from_utf8;

pub fn get_tmux_pane_height() -> Result<u16, String> {
    let mut cmd = Command::new("tmux");
    cmd.arg("display-message").arg("-p").arg("#{pane_height}");

    let process = cmd
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
