#!/bin/sh
# install.sh - install xsakura on any Linux distribution (or other Unix) that has Python 3.
#
#   sh install.sh               ask what to do: install, update or uninstall (menu)
#   sh install.sh --install     install for everyone in /usr/local (asks for your password)
#   sh install.sh --user        install only for you in ~/.local (no password needed)
#   sh install.sh --update      get the newest version (git pull) and reinstall it where it is now
#   sh install.sh --uninstall   remove it again (add --user if you installed with --user)
#
# Run it from inside the downloaded xsakura folder.
set -eu

NAME=xsakura

usage() {
    cat <<'EOF'
xsakura installer

usage: sh install.sh [options]

Run it without options to get a menu: Install, Update or Uninstall.

options:
  --install         install (the default when not asked in a terminal)
  --user            install for your user only (~/.local), no sudo needed
  --prefix DIR      install into DIR instead (programs go to DIR/bin)
  --update          get the newest version from GitHub and reinstall it where it is
                    installed now (needs a folder made with 'git clone'; for a ZIP
                    download, unzip the new ZIP and run this inside it)
  --uninstall       remove xsakura (use the same --user or --prefix as when installing)
  -y, --yes         do not ask questions (installs Python 3 if it is missing)
  -h, --help        show this help

It installs the xsakura command, its man page and tab completion for bash, zsh and fish.
EOF
}

say() { printf '%s\n' "$*"; }
info() {
    if [ -t 1 ]; then printf '\033[38;5;218m==>\033[0m %s\n' "$*"; else printf '==> %s\n' "$*"; fi
}
warn() { printf 'warning: %s\n' "$*" >&2; }
die() { printf 'error: %s\n' "$*" >&2; exit 1; }
have() { command -v "$1" >/dev/null 2>&1; }

NOARGS=0
[ $# -eq 0 ] && NOARGS=1
MODE=install
USER_MODE=0
PREFIX=
PREFIX_GIVEN=0
ASSUME_YES=0
while [ $# -gt 0 ]; do
    case $1 in
        --user) USER_MODE=1 ;;
        --prefix)
            [ $# -ge 2 ] || die "--prefix needs a directory"
            PREFIX=$2
            PREFIX_GIVEN=1
            shift
            ;;
        --prefix=*)
            PREFIX=${1#--prefix=}
            PREFIX_GIVEN=1
            ;;
        --install) MODE=install ;;
        --uninstall) MODE=uninstall ;;
        --update) MODE=update ;;
        -y | --yes) ASSUME_YES=1 ;;
        -h | --help) usage; exit 0 ;;
        *) die "unknown option: $1 (try --help)" ;;
    esac
    shift
done

HOME=${HOME:-}

# --- no options given and we are in a terminal: ask what to do
if [ "$NOARGS" = 1 ] && [ -t 0 ] && [ -t 1 ]; then
    say "xsakura installer"
    say ""
    say "  1) Install"
    say "  2) Update"
    say "  3) Uninstall"
    say "  4) Quit"
    say ""
    printf 'Choose [1-4] (Enter = 1): '
    read -r choice || choice=4
    case $choice in
        "" | 1 | [Ii]*) MODE=install ;;
        2 | [Uu][Pp]*) MODE=update ;;
        3 | [Uu]*) MODE=uninstall ;;
        *) say "Nothing done."; exit 0 ;;
    esac
    say ""
fi

# --- update / uninstall: find the copy that is installed now, so that is the one we update
OLD_VERSION=
if [ "$MODE" = update ] || [ "$MODE" = uninstall ]; then
    EXISTING=$(command -v "$NAME" 2>/dev/null || true)
    if [ -n "$EXISTING" ]; then
        OLD_VERSION=$("$EXISTING" --version 2>/dev/null || true)
    fi
    if [ -z "$PREFIX" ] && [ "$USER_MODE" = 0 ] && [ -n "$EXISTING" ]; then
        case $EXISTING in
            /usr/bin/*)
                die "xsakura at $EXISTING was installed by your package manager (the AUR or pacman), so this script will not touch it. Use: yay -Syu to update, or sudo pacman -Rns $NAME to remove"
                ;;
            */bin/"$NAME")
                PREFIX=${EXISTING%/bin/"$NAME"}
                if [ "$PREFIX" != /usr/local ] && { [ -z "$HOME" ] || [ "$PREFIX" != "$HOME/.local" ]; }; then
                    PREFIX_GIVEN=1
                fi
                ;;
        esac
    fi
fi

# --- run a command as root (only used for installing Python)
as_root() {
    if [ "$(id -u)" = 0 ]; then
        "$@"
    elif have sudo; then
        sudo "$@"
    elif have doas; then
        doas "$@"
    else
        return 1
    fi
}

# --- pick where to install
SUDO=
if [ -z "$PREFIX" ]; then
    if [ "$USER_MODE" = 1 ]; then
        [ -n "$HOME" ] || die "HOME is not set, cannot use --user"
        PREFIX=$HOME/.local
    elif [ "$(id -u)" = 0 ]; then
        PREFIX=/usr/local
    elif have sudo; then
        PREFIX=/usr/local
        SUDO=sudo
    elif have doas; then
        PREFIX=/usr/local
        SUDO=doas
    else
        warn "sudo and doas were not found, installing for your user only"
        [ -n "$HOME" ] || die "HOME is not set"
        PREFIX=$HOME/.local
    fi
fi

nearest_dir() {
    d=$1
    while [ ! -d "$d" ]; do d=$(dirname "$d"); done
    printf '%s' "$d"
}
if [ -z "$SUDO" ] && [ "$(id -u)" != 0 ] && [ ! -w "$(nearest_dir "$PREFIX")" ]; then
    if have sudo; then SUDO=sudo
    elif have doas; then SUDO=doas
    else die "cannot write to $PREFIX and sudo/doas were not found (try: sh install.sh --user)"
    fi
fi

run() {
    if [ -n "$SUDO" ]; then "$SUDO" "$@"; else "$@"; fi
}

BIN=$PREFIX/bin
MAN=$PREFIX/share/man/man1
BASH_DIR=$PREFIX/share/bash-completion/completions
ZSH_DIR=$PREFIX/share/zsh/site-functions
if [ -n "$HOME" ] && [ "$PREFIX" = "$HOME/.local" ]; then
    FISH_DIR=${XDG_CONFIG_HOME:-$HOME/.config}/fish/completions
else
    FISH_DIR=$PREFIX/share/fish/vendor_completions.d
fi

# --- uninstall
if [ "$MODE" = uninstall ]; then
    info "Removing xsakura from $PREFIX"
    removed=0
    for f in "$BIN/$NAME" "$MAN/$NAME.1" "$BASH_DIR/$NAME" "$ZSH_DIR/_$NAME" "$FISH_DIR/$NAME.fish"; do
        if [ -e "$f" ]; then
            run rm -f "$f"
            say "  removed $f"
            removed=1
        fi
    done
    if [ "$removed" = 0 ]; then
        say "Nothing to remove in $PREFIX. If you installed with --user or --prefix, use the same option here."
    else
        info "Done. xsakura is removed."
    fi
    exit 0
fi

# --- make sure Python 3 is there
python_ok() {
    have python3 && python3 -c 'import sys; sys.exit(0 if sys.version_info >= (3, 6) else 1)' 2>/dev/null
}

install_python() {
    if have pacman; then
        as_root pacman -S --needed --noconfirm python
    elif have apt-get; then
        as_root apt-get update && as_root apt-get install -y python3
    elif have dnf; then
        as_root dnf install -y python3
    elif have zypper; then
        as_root zypper --non-interactive install python3
    elif have apk; then
        as_root apk add python3
    elif have xbps-install; then
        as_root xbps-install -y python3
    elif have emerge; then
        as_root emerge --ask=n dev-lang/python
    else
        return 1
    fi
}

if ! python_ok; then
    warn "Python 3.6 or newer is required but was not found."
    answer=n
    if [ "$ASSUME_YES" = 1 ]; then
        answer=y
    elif [ -t 0 ]; then
        printf 'Install Python 3 now with your package manager? [Y/n] '
        read -r answer || answer=n
        case $answer in "") answer=y ;; esac
    fi
    case $answer in
        y | Y | yes | YES) ;;
        *) die "please install Python 3 (package name: python3 or python) and run this again" ;;
    esac
    install_python || die "could not install Python automatically. Install Python 3 yourself and run this again"
    python_ok || die "Python 3.6 or newer is still not available"
fi

# --- find the files next to this script
SRC=$(cd "$(dirname "$0")" && pwd -P)
[ -f "$SRC/$NAME" ] || die "cannot find the '$NAME' file next to install.sh. Download or clone the whole repository and run it from inside that folder"

# --- update: download the newest files first (only if this folder is its own git clone)
if [ "$MODE" = update ]; then
    if have git && [ "$(git -C "$SRC" rev-parse --show-toplevel 2>/dev/null)" = "$SRC" ]; then
        info "Downloading the newest version from GitHub"
        git -C "$SRC" pull --ff-only ||
            die "git pull failed. If you edited files in this folder, run 'git status' in it to see them, or clone the repository again"
    else
        info "This folder is not a git clone, so it cannot download updates by itself."
        say "  Updating from the files in this folder. For the newest version, download the"
        say "  latest ZIP from https://github.com/77nyan/xsakura or run: git clone https://github.com/77nyan/xsakura.git"
    fi
fi

put() { # file, destination dir, destination name, mode
    if [ ! -f "$SRC/$1" ]; then
        warn "skipping $1 (not found)"
        return 0
    fi
    run install -d "$2"
    run install -m "$4" "$SRC/$1" "$2/$3"
}

info "Installing xsakura to $PREFIX"
put "$NAME" "$BIN" "$NAME" 755
put "$NAME.1" "$MAN" "$NAME.1" 644
put "$NAME.bash" "$BASH_DIR" "$NAME" 644
put "_$NAME" "$ZSH_DIR" "_$NAME" 644
put "$NAME.fish" "$FISH_DIR" "$NAME.fish" 644

NEW_VERSION=$("$BIN/$NAME" --version 2>/dev/null) || die "the installed program did not start. Run '$BIN/$NAME --version' to see the error"

if [ "$MODE" != update ]; then
    info "Done! Type: xsakura"
elif [ -z "$OLD_VERSION" ]; then
    info "No earlier xsakura was found, so this is a fresh install: $NEW_VERSION"
elif [ "$OLD_VERSION" != "$NEW_VERSION" ]; then
    info "Updated: $OLD_VERSION -> $NEW_VERSION"
else
    info "You are up to date: $NEW_VERSION"
fi
case ":${PATH:-}:" in
    *":$BIN:"*) ;;
    *)
        warn "$BIN is not in your PATH, so the 'xsakura' command will not be found yet."
        say "  Add this line to ~/.bashrc or ~/.zshrc, then open a new terminal:"
        say "    export PATH=\"$BIN:\$PATH\""
        ;;
esac
say "  Try:  xsakura            (grow a tree, press q to quit)"
say "        xsakura -mini      (a shorter, wider tree)"
say "        xsakura --help     (all colors and options)"
UNINSTALL="sh install.sh --uninstall"
if [ "$PREFIX_GIVEN" = 1 ]; then
    UNINSTALL="$UNINSTALL --prefix $PREFIX"
elif [ "$PREFIX" = "$HOME/.local" ]; then
    UNINSTALL="$UNINSTALL --user"
fi
say "  Remove it later with:  $UNINSTALL"
