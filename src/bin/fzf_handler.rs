use itertools::Itertools;
use pluckrs::tmux_utils;
use regex::Regex;
use std::{
    io::Write,
    process::{Command, Stdio},
    str::from_utf8,
};
// Note: There is a reload mode for fzf, maybe it's faster??
// EXIT STATUS for fzf
//        0      Normal exit
//        1      No match
//        2      Error
//        126    Permission denied error from become action
//        127    Invalid shell command for become action
//        130    Interrupted with CTRL-C or ESC

// Next step: Make it possible to insert the selected text
// Next step: Make a struct that handles a clipboard configuration etc

fn copy_into_clipboard(text: &str, clip_tool: &str) -> Result<(), String> {
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

fn insert_text(text: &str, tmux_pane: &str) -> Result<(), String> {
    Command::new("tmux")
        .arg("send-keys")
        .arg("-t")
        .arg(tmux_pane)
        .arg(text)
        .output()
        .map_err(|e| e.to_string())?;

    Ok(())
}

fn get_filtered_data_from_lines(lines: &Vec<&str>, regex_pattern: &Regex) -> String {
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
        .unique()
        .join("\n");
    return result;
}

fn main() -> Result<(), String> {
    let backward_history: u32 = 500000;
    let filter_button = "ctrl-f";
    let copy_button = "enter";
    let insert_button = "tab";
    let clip_tool = "pbcopy"; // Only MacOS support for now :^)
    let tmux_pane = std::env::var("TMUX_PANE").map_err(|e| e.to_string())?;

    let pane_height = tmux_utils::get_tmux_pane_height()
        .map_err(|e| format!("Failed to read pane height! Error: {:?}", e))?;

    let original_buffer_contents =
        tmux_utils::get_tmux_buffer_contents(pane_height, backward_history)?;

    // Remove empty lines and duplicates
    let original_buffer_lines = original_buffer_contents
        .lines()
        .filter(|l| !l.is_empty())
        .unique()
        .collect::<Vec<&str>>();

    let path_regex =
        Regex::new(r#"(?:[ \t\n"\(\[<':]|^)(~|/)?([-~a-zA-Z0-9_+,\-.]+/[^ \t\n\r|:"'$%&)>\]]*)"#)
            .unwrap();

    let url_regex = Regex::new(
        r#"(https?://|git@|git://|ssh://|s?ftp://|file:///)([a-zA-Z0-9?=%/_.:,;~@!#$&()*+\-]*)"#,
    )
    .unwrap();

    let line_regex = Regex::new(r#"^.*$"#).unwrap();

    // Finding all regex matches across lines. Allows multiple matches per line

    let modes = vec!["line", "path", "url"];
    let mut modes_iter = modes.iter().cycle();
    let mut mode: &str = modes_iter.next().unwrap();

    let mut chosen_regex = &path_regex;
    if mode == "path" {
        chosen_regex = &path_regex;
    } else if mode == "url" {
        chosen_regex = &url_regex;
    } else if mode == "line" {
        chosen_regex = &line_regex;
    }
    let mut buffer_contents = get_filtered_data_from_lines(&original_buffer_lines, &chosen_regex);

    // Later plans: implement direct access...
    // ctrl-1 -> 1
    // ctrl-2 -> 2
    // ctrl-3 -> 3
    // etc.

    let mut query = String::from("");

    loop {
        let mut fzf_cmd = Command::new("fzf");
        fzf_cmd.arg(format!("--expect={}", filter_button));
        fzf_cmd.arg(format!("--expect={}", copy_button));
        fzf_cmd.arg(format!("--expect={}", insert_button));
        fzf_cmd.arg("--expect=ctrl-g,esc"); // Adding basic buttons to ensure always capturing a key
        fzf_cmd.arg("--print-query"); // Get the query when fzf finishes
        fzf_cmd.arg(format!("--query={}", query)); // Inject the previous query
        fzf_cmd.stdin(Stdio::piped());
        fzf_cmd.stdout(Stdio::piped());

        let header = format!(
            "PLUCKRS - mode: {} - Change mode using {}",
            mode, filter_button
        );
        fzf_cmd.arg(format!("--header={}", header));

        let mut fzf_process_handle = fzf_cmd.spawn().map_err(|e| e.to_string())?;
        let mut fzf_stdin_handle = fzf_process_handle
            .stdin
            .take()
            .ok_or("Unable to get stdin handle from fzf process")?;

        fzf_stdin_handle
            .write_all(buffer_contents.as_bytes())
            .map_err(|e| format!("Failed to write buffer to fzf stdin with error {}", e))?;

        drop(fzf_stdin_handle);
        let output = fzf_process_handle.wait_with_output().unwrap();

        // If the user presses escape or ctrl+c
        let output_code = output.status.code().unwrap();
        if output_code == 130 {
            println!("User exited!");
            break;
        }

        if output_code == 2 {
            println!("fzf exited with error code 2!");
            break;
        }

        // Parsing the output
        let stdout_str = from_utf8(&output.stdout).unwrap().trim();
        let fzf_output_list: Vec<&str> = stdout_str.split("\n").collect();
        println!("fzf_output_list: {:?}", fzf_output_list);

        // The output has the format [query, key_press, match], but only if it has an actual
        // value for them, i.e. the list could be [key_press, match] if no query was present
        // or [query, key_press] if there was no match
        let mut key_press: String = String::from("");
        let mut selection: String = String::from("");

        // If the exit code is 0, we know there is a match. Thus, if the length is 2
        // we know that the output has the form [key_press, match]

        if fzf_output_list.len() == 3 {
            query = fzf_output_list[0].to_string();
            key_press = fzf_output_list[1].to_string();
            selection = fzf_output_list[2].to_string();
        } else if output_code == 0 && fzf_output_list.len() == 2 {
            key_press = fzf_output_list[0].to_string();
            selection = fzf_output_list[1].to_string();
        } else if output_code == 1 && fzf_output_list.len() == 2 {
            query = fzf_output_list[0].to_string();
            key_press = fzf_output_list[1].to_string();
        } else if fzf_output_list.len() == 1 {
            key_press = fzf_output_list[0].to_string();
        }

        if key_press == filter_button {
            mode = modes_iter.next().unwrap();
            chosen_regex = match mode {
                "path" => &path_regex,
                "url" => &url_regex,
                "line" => &line_regex,
                _ => &line_regex,
            };
            buffer_contents = get_filtered_data_from_lines(&original_buffer_lines, chosen_regex);
        } else if key_press == copy_button {
            copy_into_clipboard(&selection, clip_tool)?;
            break;
        } else if key_press == insert_button {
            insert_text(&selection, &tmux_pane)?;
            break;
        }

        println!("query: {}", query);
        println!("key_press: {}", key_press);
        println!("selection: {}", selection);
        println!("status: {}", output.status);
        println!("stdout: {}", stdout_str);
        println!("stderr: {:?}", output.stderr);
    }

    // fzf_cmd.stdin(cfg)
    // fzf_cmd.stdin.a
    Ok(())
}
