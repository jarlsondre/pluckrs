#!/bin/sh
CURRENT_SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

cd $CURRENT_SCRIPT_DIR

# Build the project
cargo build --release

# Passing the pane_id as an env variable for downstream access
tmux bind-key W run-shell "TMUX_PANE=#{pane_id} $CURRENT_SCRIPT_DIR/target/release/pluckrs"
