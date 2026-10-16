# xsakura

An animated pixel-art cherry blossom tree for your terminal. It sprouts from the
bottom edge, branches out, bursts into bloom, then sheds petals that drift in
the wind. 13 colors, a transparent background and no dependencies except
Python. Nothing else is drawn, just the tree. Made for ricing and screenshots.

```
xsakura            # grow the classic pink tree
xsakura -red       # a red one
xsakura -green     # a green one
```

The background stays transparent, so it looks right on top of your wallpaper.
It is a single Python script with no dependencies besides `python`.

## Colors

`-sakura` (default), `-pink`, `-red`, `-orange`, `-yellow`, `-gold`, `-green`,
`-cyan`, `-blue`, `-purple`, `-magenta`, `-white`, `-rainbow`, or any color with
`-hex ff8800`. Run `xsakura --list` to see them.

## Options

| option | what it does |
| --- | --- |
| `-s, --seed N` | reproducible tree (same seed and window size = same tree) |
| `-p, --print` | print the finished tree and exit, great for screenshots |
| `-i, --instant` | skip the growing animation |
| `-t, --time SEC` | quit after SEC seconds |
| `--cycle SEC` | grow a new tree SEC seconds after the last one finished |
| `--speed X` | animation speed (default 1.0) |
| `--grow SEC` | how long the tree needs to grow (default 9) |
| `--petals N` | number of falling petals |
| `--wind X` | wind strength, negative blows left |
| `--scale X` | tree size multiplier |
| `--width N`, `--height N` | size in cells instead of the terminal size |
| `--truecolor`, `--256` | force the color mode |

Keys while it runs: `q` / `Esc` quit, `r` grow a new tree, `space` pause.

## Ricing ideas

```sh
# screensaver: a new tree every 30 seconds after the last one is done
xsakura --cycle 30

# an already grown tree for your wallpaper terminal
xsakura -i -s 12

# greet yourself in ~/.zshrc or ~/.bashrc
xsakura -p --height 14

# hyprland / sway: put it in a terminal on a scratch workspace
exec = kitty --class xsakura -e xsakura --cycle 60
```

## Requirements

A terminal with 24-bit color and a font that has the half block characters
(`▀ ▄ █`), which is practically every modern terminal: kitty, alacritty, foot,
wezterm, konsole, gnome-terminal and so on. 256-color terminals work with
slightly less smooth colors.

## License

MIT
