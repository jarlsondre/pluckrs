#!/bin/sh
CURRENT_SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$CURRENT_SCRIPT_DIR" || exit 1

cargo build --release

# The pane_id is needed for tmux to know where to read the lines from
tmux bind-key W run-shell "TMUX_PANE=#{pane_id} $CURRENT_SCRIPT_DIR/target/release/pluckrs"
