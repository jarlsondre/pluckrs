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

## Implemented Features

- Show lines from current `tmux` pane and search through them with `fzf`
- Filter by path (kinda wonky still, regex might need some work)
- Filter by url (very wonky still, regex definitely needs some work)
- Copy selection
 - Only supports `pbcopy`, i.e. MacOS, at the moment
- Insert selection

## Planned Features

- Configure plugin in configuration file
- Configure plugin in `.tmux.conf`
