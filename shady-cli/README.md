# Shady-Cli

It's a [cava] inspired audio visualizer for the terminal which I developed to have a visualization
of [shady-audio].

# Demo

[![Demo video](https://img.youtube.com/vi/FnB8QZckJlM/maxresdefault.jpg)](https://www.youtube.com/watch?v=FnB8QZckJlM)

# Usage

The keybindings are:

- `+` to increase the width of the bars which also decreases the amount of bars since the space becomes smaller
- `-` to decrease the width of the bars which also increase the amount of bars since the space becomes bigger
- `q` to quit

There are also some arguments. Take a look at the help page (`-h` or `--help`).

### `nix` with flakes

```bash
nix run github:TornaxO7/shady#shady-cli -- <shady-toy args>
```

#### Example

- `nix run github:TornaxO7/shady#shady-cli -- --color red` to start the visualizer with red bars.

### Build it yourself

You just need `alsa-lib` and [rust], then navigate into this directory and execute

```bash
cargo run --release -- <shady-cli args>
```

#### Example

- `cargo run --release -- --bar-width 3` start `shady-cli` with an initial bar width of `3`

# Other notes

`shady-cli` is not as mature as [cava]. If you want a solid experience, then [cava] is the way to go.

[cava]: https://github.com/karlstav/cava
[shady-audio]: https://github.com/TornaxO7/shady/tree/main/shady-audio
[rust]: https://www.rust-lang.org/
