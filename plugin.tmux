#!/bin/sh
CURRENT_SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

tmux bind-key W run-shell "TMUX_PANE=#{pane_id} $CURRENT_SCRIPT_DIR/target/release/pluckrs"
