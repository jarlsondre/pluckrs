# pluckrs
A rewrite of the popular [Extrakto](https://github.com/laktak/extrakto) plugin for `tmux`.
There are three reasons for this project:
- Learning Rust
- Performance
- Maintainability

As it stands right now, the plugin loads considerably faster than than Extrakto. This is most
likely due to the fact that you're launching a compiled binary instead of Python code.
Additionally, it seems to filter quite a bit faster. This is most likely just due to Rust
being, well, a lot faster than Python (obviously).

Keep in mind that this is currently in development. This, there are a ton of missing features.

## Requirements
- `tmux` (obviously)
- `fzf`
- `cargo` (`Rust`)
