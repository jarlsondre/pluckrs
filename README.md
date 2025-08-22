# pluckrs
A rewrite of the popular [Extrakto](https://github.com/laktak/extrakto) plugin for `tmux`.
There are three reasons for this project:
- Learning Rust
- Performance
- Maintainability

As it stands right now, the plugin loads considerably faster than than Extrakto. This is most
likely due to the fact that you're launching a compiled binary instead of Python code.
Additionally, it seems to filter quite a bit faster. This is most likely just due to Rust
being, well, a lot faster than Python.

Keep in mind that this is currently in development. Thus, there are a ton of missing features.

## Requirements

- `tmux` (obviously)
- `fzf`
- `cargo` (`Rust`)

## Configuration

To configure the application, you have to create a file with the path 
`~/.config/pluckrs/config.toml`. You can check out the `sample_config.toml` for information
on how to structure your config.

## Implemented Features

- Show lines from current `tmux` pane and search through them with `fzf`
- Filter by path (regex completely rewritten seems to work well now)
- Filter by url (wonky still, regex definitely needs some work)
- Copy selection (only supports `pbcopy` at the moment, but user can specify in config)
- Insert selection

## Planned Improvements

- Tests (!!!)
- Remove the need for a configuration file
- Adding `all` filter that combines all other filters
- Choose whether to use `tmux popup` or something else
- Configure plugin in `.tmux.conf`
