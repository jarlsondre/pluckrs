use pluckrs::tmux_utils;
use pluckrs::utils;

use itertools::Itertools;
use regex::Regex;
use std::str::from_utf8;

// Note: There is a reload mode for fzf, maybe it's faster??
// Should probably validate the regexes inside the configuration
// At least that the modes match the actual values inside of the hashmap

fn main() -> Result<(), String> {
    let configuration = utils::get_home_config_file()?;
    let regex_map = configuration.regexes;
    let min_length = configuration.general.min_length;
    let regex_order = configuration.general.regex_order;
    let mut regex_iter = regex_order.iter().cycle();

    let backward_history = configuration.general.backward_history;
    let clip_tool = configuration.general.clip_tool;

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
    let mut mode = regex_iter
        .next()
        .ok_or_else(|| "Failed to read mode from regex_iter")?;
    let mut chosen_regex_str = &regex_map[mode];
    let mut chosen_regex = Regex::new(&chosen_regex_str).map_err(|e| {
        format!(
            "Creating regex from {} failed! Error: {:?}",
            chosen_regex_str, e
        )
    })?;
    let mut buffer_contents =
        utils::get_filtered_data_from_lines(&original_buffer_lines, &chosen_regex, min_length);

    let mut query = String::from("");

    loop {
        let fzf_output = utils::launch_fzf(
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

            buffer_contents = utils::get_filtered_data_from_lines(
                &original_buffer_lines,
                &chosen_regex,
                min_length,
            );
        } else if key_press == copy_button {
            utils::copy_into_clipboard(&selection, &clip_tool)?;
            break;
        } else if key_press == insert_button {
            utils::insert_text(&selection, &tmux_pane)?;
            break;
        } else if key_press == "esc" {
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
