use pluckrs::tmux_utils;
use std::{
    io::Write,
    process::{Command, Stdio},
};

fn main() -> Result<(), String> {
    let pane_height = tmux_utils::get_tmux_pane_height()
        .map_err(|e| format!("Failed to read pane height! Error: {:?}", e))?;
    println!("Read the pane height: {}", pane_height);

    let backward_history = 200;
    let buffer_contents = tmux_utils::get_tmux_buffer_contents(pane_height, backward_history)?;
    println!("Read the contents of the buffer! Contents: {}", buffer_contents);

    let mut fzf_cmd = Command::new("fzf");
    fzf_cmd.stdin(Stdio::piped());
    let mut fzf_process_handle = fzf_cmd.spawn().map_err(|e| e.to_string())?;

    let mut fzf_stdin_handle = fzf_process_handle
        .stdin
        .take()
        .ok_or("Unable to get stdin handle from fzf process")?;

    fzf_stdin_handle
        .write_all(buffer_contents.as_bytes())
        .map_err(|e| format!("Failed to write buffer to fzf stdin with error {}", e))?;

    fzf_process_handle.wait();
    // fzf_cmd.stdin(cfg)
    // fzf_cmd.stdin.a
    Ok(())
}
