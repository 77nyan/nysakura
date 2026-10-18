# xsakura

An animated pixel-art cherry blossom tree for your terminal. It sprouts from the
bottom edge, branches out, bursts into bloom, then sheds petals that drift in
the wind. 13 colors, a transparent background and no dependencies except
Python. Nothing else is drawn, just the tree. Made for ricing and screenshots.

```sh
xsakura            # a tall pink tree, made for full-height terminals
xsakura -mini      # a shorter, wider tree
xsakura -red       # a red one
xsakura -green     # a green one
```

<!-- Add screenshots: save them in this folder (for example screenshot.png) and add a line like: ![xsakura](screenshot.png) -->

## Install

The install script works the same on **every Linux distribution, including Arch
Linux** and Arch-based ones such as Omarchy, Manjaro and EndeavourOS. You need
Python 3.6 or newer. Most distributions already have it, and the script offers to
install it if it is missing.

```sh
git clone https://github.com/77nyan/xsakura.git
cd xsakura
sh install.sh
```

Running `sh install.sh` with no options shows a small menu: **1) Install**,
**2) Update**, **3) Uninstall**, **4) Quit**. Type a number and press Enter.

No git? Click the green **Code** button on this page, choose **Download ZIP**,
unzip it, open a terminal in the unzipped folder, and run `sh install.sh`.

When you choose Install, the script asks for your password (sudo) and installs for
everyone in `/usr/local`. It installs the `xsakura` command, its man page and tab completion
for bash, zsh and fish.

| Command | What it does |
| --- | --- |
| `sh install.sh` | Show the menu (Install, Update, Uninstall) |
| `sh install.sh --install` | Install system-wide in `/usr/local` |
| `sh install.sh --user` | Install only for you in `~/.local`, no password needed |
| `sh install.sh --update` | Get the newest version and reinstall it (see [Update](#update)) |
| `sh install.sh --prefix DIR` | Install into `DIR` instead |
| `sh install.sh --uninstall` | Remove it again (add `--user` or `--prefix DIR` if you used one) |
| `sh install.sh -y` | Do not ask questions, and install Python 3 if it is missing |

If you used `--user` and the shell says `xsakura: command not found`, add
`~/.local/bin` to your PATH by putting this line in `~/.bashrc` or `~/.zshrc`, then
open a new terminal:

```sh
export PATH="$HOME/.local/bin:$PATH"
```

In zsh, tab completion from a `--user` install needs this line in `~/.zshrc`
before `compinit`:

```sh
fpath+=(~/.local/share/zsh/site-functions)
```

### Just try it without installing

Inside the downloaded folder:

```sh
python3 xsakura
```

### Arch Linux: AUR package (coming soon)

An AUR version is coming soon. When it is published, Arch users will also be able
to install xsakura with one command:

```sh
yay -S xsakura
# or
paru -S xsakura
```

Note the capital `S`. A lowercase `-s` does something else.

Until then, use the install script above. It works exactly the same on Arch.

<details>
<summary>I don't have yay or paru. How do I set up the AUR?</summary>

The AUR (Arch User Repository) is a collection of community-made packages. There
is no switch to turn on in Arch itself. You need two things: `git` and the
`base-devel` group (the build tools), plus an "AUR helper" such as `yay` or
`paru` that downloads and builds AUR packages for you.

1. Check whether you already have one. EndeavourOS and CachyOS usually ship with a
   helper:

   ```sh
   yay --version
   paru --version
   ```

2. If both say `command not found`, install the build tools:

   ```sh
   sudo pacman -S --needed base-devel git
   ```

3. Install a helper. Pick one. `-bin` means a prebuilt version, which installs fast:

   ```sh
   # yay
   git clone https://aur.archlinux.org/yay-bin.git
   cd yay-bin
   makepkg -si
   ```

   ```sh
   # paru
   git clone https://aur.archlinux.org/paru-bin.git
   cd paru-bin
   makepkg -si
   ```

4. Once xsakura is on the AUR, `yay -S xsakura` (or `paru -S xsakura`) works.

**Manjaro:** open Pamac, go to **Preferences**, then **Third Party**, and turn on
**Enable AUR support**. Then search for `xsakura` in Pamac, or run
`pamac build xsakura` in a terminal.

**No helper at all:** once the package is on the AUR, you can build it by hand:

```sh
git clone https://aur.archlinux.org/xsakura.git
cd xsakura
makepkg -si
```

</details>

### Windows

xsakura is made for Linux terminals. On Windows, install WSL (`wsl --install` in
PowerShell), open the Linux terminal, and follow the install steps above.

## Update

To get the newest version, go into the xsakura folder you cloned and run:

```sh
cd xsakura
sh install.sh --update
```

Or run `sh install.sh` and choose **2) Update** in the menu.

This downloads the newest files from GitHub (`git pull`) and reinstalls xsakura in
the same place it is installed now, so you do not have to remember whether you used
`--user`. It tells you what changed, for example
`Updated: xsakura 1.1.0 -> xsakura 1.2.0`. You can check your version any time with
`xsakura --version`.

- **Downloaded the ZIP instead of cloning?** Download the newest ZIP from this page,
  unzip it, and run `sh install.sh --update` inside the new folder. It finds your
  existing install and updates it.
- **Deleted the folder?** Clone it again with
  `git clone https://github.com/77nyan/xsakura.git`, then run
  `sh install.sh --update` inside it.
- **`git pull` says you have local changes?** You edited a file in the folder. Run
  `git status` to see which, undo or save your edits, and try again.
- **Installed from the AUR package (once it is available)?** Update it like any
  other package with `yay -Syu`. `sh install.sh --update` will not touch a copy that
  your package manager installed, and tells you this.

## Usage

### Colors

Give a color as a single-dash option:

`-sakura` (default), `-pink`, `-red`, `-orange`, `-yellow`, `-gold`, `-green`,
`-cyan`, `-blue`, `-purple`, `-magenta`, `-white`, `-rainbow`, or any color with
`-hex ff8800`. Run `xsakura --list` to see them.

### Tree size: tall by default, `-mini` for a shorter one

By default xsakura grows a tall tree with a long trunk and a slim crown. It fills
about 90% of the terminal height and stays inside the window, so it looks best in
a full-height terminal.

`-mini` grows the shorter, wider tree with a short trunk and a round crown. It
suits a short or wide terminal, but it looks short in a tall, narrow one.

```sh
xsakura                    # tall pink tree (the default)
xsakura -mini              # the shorter, wider tree
xsakura -mini -blue        # works with every color
xsakura -s 12 -i           # a finished tree, the same one every time
```

### Options

| option | what it does |
| --- | --- |
| `-mini` | shorter, wider tree (the default tree is tall and slim) |
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

- Python 3.6 or newer (tested with 3.13)
- A terminal with 24-bit color and a font that has the half block characters
  (`▀ ▄ █`), which is practically every modern terminal: kitty, alacritty, foot,
  wezterm, konsole, gnome-terminal and so on. 256-color terminals work with slightly
  less smooth colors.

## Uninstall

First find out how it was installed:

```sh
command -v xsakura
```

| It prints | You installed it with | Go to |
| --- | --- | --- |
| `/usr/bin/xsakura` | the AUR package (once it is available) | [Installed from the AUR](#installed-from-the-aur-once-it-is-available) |
| `/usr/local/bin/xsakura` | `sh install.sh` | [Installed with the install script](#installed-with-the-install-script) |
| `~/.local/bin/xsakura` | `sh install.sh --user` | [Installed with the install script](#installed-with-the-install-script) |

xsakura does not create any settings or config files, so removing the program
leaves nothing else behind.

### Installed from the AUR (once it is available)

```sh
sudo pacman -Rns xsakura
```

`yay -Rns xsakura` and `paru -Rns xsakura` do the same thing. `-R` removes the
package, `-n` also removes its leftover config backups, and `-s` removes
dependencies that nothing else needs. It will not remove Python if other programs
use it.

### Installed with the install script

Open a terminal in the xsakura folder you downloaded or cloned, run
`sh install.sh` and choose **3) Uninstall**. It finds where xsakura is installed by
itself. You can also use the option directly:

```sh
sh install.sh --uninstall              # installed with: sh install.sh
sh install.sh --uninstall --user       # installed with: sh install.sh --user
sh install.sh --uninstall --prefix DIR # installed with: sh install.sh --prefix DIR
```

It removes the command, the man page and the tab completion, and lists each file
it deleted. If you already deleted the folder, clone it again
(`git clone https://github.com/77nyan/xsakura.git`) or remove the files by hand:

```sh
# installed with: sh install.sh
sudo rm -f /usr/local/bin/xsakura \
  /usr/local/share/man/man1/xsakura.1 \
  /usr/local/share/bash-completion/completions/xsakura \
  /usr/local/share/zsh/site-functions/_xsakura \
  /usr/local/share/fish/vendor_completions.d/xsakura.fish

# installed with: sh install.sh --user
rm -f ~/.local/bin/xsakura \
  ~/.local/share/man/man1/xsakura.1 \
  ~/.local/share/bash-completion/completions/xsakura \
  ~/.local/share/zsh/site-functions/_xsakura \
  ~/.config/fish/completions/xsakura.fish
```

### Just ran it from the folder

If you only ran `python3 xsakura` without installing, there is nothing to remove.
Delete the folder if you do not want the files anymore.

## Credits

Made by Jakub Ziaja ([@77nyan](https://github.com/77nyan)).

## License

[MIT](LICENSE)
