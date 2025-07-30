use dirs::home_dir;
use itertools::Itertools;
use pluckrs::config;
use pluckrs::tmux_utils;
use regex::Regex;
use std::{
    io::Write,
    process::{Command, Output, Stdio},
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

// Should probably validate the regexes inside the configuration

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

fn launch_fzf(
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
        "PLUCKRS | mode: {} | Toggle filter: {} | Copy: {} | Insert: {}",
        mode, filter_button, copy_button, insert_button
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

fn main() -> Result<(), String> {
    let config_file_path = home_dir()
        .unwrap()
        .join(".config")
        .join("pluckrs")
        .join("config.toml");
    let configuration = config::read_config(config_file_path.to_str().unwrap()).unwrap();
    let regex_map = configuration.regexes;
    let regex_order = configuration.general.regex_order;
    let mut regex_iter = regex_order.iter().cycle();
    // let mut regex_iter = regex_map.iter().cycle();

    let backward_history = configuration.general.backward_history;
    let clip_tool = configuration.general.clip_tool.unwrap();

    let copy_button = configuration.keybinds.copy;
    let filter_button = configuration.keybinds.filter;
    let insert_button = configuration.keybinds.insert;

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

    // let (mut mode, mut chosen_regex_str) = regex_iter.next().unwrap();
    let mut mode = regex_iter.next().unwrap();
    let mut chosen_regex_str = &regex_map[mode];
    let mut chosen_regex = Regex::new(&chosen_regex_str).unwrap();
    let mut buffer_contents = get_filtered_data_from_lines(&original_buffer_lines, &chosen_regex);

    // Later plans: implement direct access...
    // ctrl-1 -> 1
    // ctrl-2 -> 2
    // ctrl-3 -> 3
    // etc.

    let mut query = String::from("");

    loop {
        let fzf_output = launch_fzf(
            &query,
            &mode,
            &buffer_contents,
            &filter_button,
            &copy_button,
            &insert_button,
        )
        .unwrap();

        // If the user presses escape or ctrl+c
        let output_code = fzf_output.status.code().unwrap();
        if output_code == 130 {
            println!("User exited!");
            break;
        }

        if output_code == 2 {
            println!("fzf exited with error code 2!");
            break;
        }

        // Parsing the output
        let stdout_str = from_utf8(&fzf_output.stdout).unwrap().trim();
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
            mode = regex_iter.next().unwrap();
            chosen_regex_str = &regex_map[mode];
            chosen_regex = Regex::new(&chosen_regex_str).unwrap();

            buffer_contents = get_filtered_data_from_lines(&original_buffer_lines, &chosen_regex);
        } else if key_press == copy_button {
            copy_into_clipboard(&selection, &clip_tool)?;
            break;
        } else if key_press == insert_button {
            insert_text(&selection, &tmux_pane)?;
            break;
        }

        println!("query: {}", query);
        println!("key_press: {}", key_press);
        println!("selection: {}", selection);
        println!("status: {}", fzf_output.status);
        println!("stdout: {}", stdout_str);
        println!("stderr: {:?}", fzf_output.stderr);
    }

    // fzf_cmd.stdin(cfg)
    // fzf_cmd.stdin.a
    Ok(())
}
