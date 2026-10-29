#!/bin/sh
# nysakura installer, styled after the AerynOS "Lichen" installer.
#
#   sudo ./install.sh
#
# A small menu: Install, Update, Uninstall. It checks and (with your OK) installs the
# build tools, compiles nysakura, and copies it to /usr/local. Nothing else is touched.
# Plain POSIX sh, no options to remember.
set -u

NAME=nysakura
PREFIX=/usr/local
BIN=$PREFIX/bin
MAN=$PREFIX/share/man/man1
BASH_DIR=$PREFIX/share/bash-completion/completions
ZSH_DIR=$PREFIX/share/zsh/site-functions
FISH_DIR=$PREFIX/share/fish/vendor_completions.d
DOC_DIR=$PREFIX/share/doc/$NAME
LIC_DIR=$PREFIX/share/licenses/$NAME
MIN_RUST_MINOR=63

HERE=$(cd "$(dirname "$0")" 2>/dev/null && pwd -P) || HERE=.
ESC=$(printf '\033')

# ---------------------------------------------------------------- look (Lichen / clack style)
C_OFF= C_BAR= C_CYAN= C_GREEN= C_YELLOW= C_RED= C_BLUE= C_DIM= C_BOLD= C_PILL=
if [ -t 1 ] && [ -z "${NO_COLOR:-}" ] && [ "${TERM:-dumb}" != dumb ]; then
    C_OFF="${ESC}[0m"
    C_BAR="${ESC}[90m"
    C_CYAN="${ESC}[36m"
    C_GREEN="${ESC}[32m"
    C_YELLOW="${ESC}[33m"
    C_RED="${ESC}[31m"
    C_BLUE="${ESC}[34m"
    C_DIM="${ESC}[2m"
    C_BOLD="${ESC}[1m"
    C_PILL="${ESC}[1;30;48;5;218m"
fi

if [ "${TERM:-}" = linux ] || [ "${TERM:-}" = dumb ]; then
    S_BAR='|' S_START='+' S_END='+' S_ACTIVE='*' S_SUBMIT='o' S_WARN='!' S_ERR='x' S_INFO='*'
    S_RON='*' S_ROFF='o' S_OK='+' S_BAD='x' S_DASH='-' S_TR='+' S_BL='+' S_BR='+'
    SPIN_FRAMES='| / - \'
else
    S_BAR='│' S_START='┌' S_END='└' S_ACTIVE='◆' S_SUBMIT='◇' S_WARN='▲' S_ERR='■' S_INFO='●'
    S_RON='●' S_ROFF='○' S_OK='✔' S_BAD='✘' S_DASH='─' S_TR='╮' S_BL='├' S_BR='╯'
    SPIN_FRAMES='◒ ◐ ◓ ◑'
fi

rep() { # rep COUNT STRING
    n=$1
    while [ "$n" -gt 0 ]; do
        printf '%s' "$2"
        n=$((n - 1))
    done
}
spaces() { rep "$1" ' '; }

bar() { printf '%s%s%s\n' "$C_BAR" "$S_BAR" "$C_OFF"; }
line() { printf '%s%s%s  %s\n' "$C_BAR" "$S_BAR" "$C_OFF" "$*"; }
dimline() { printf '%s%s%s  %s%s%s\n' "$C_BAR" "$S_BAR" "$C_OFF" "$C_DIM" "$*" "$C_OFF"; }
intro() { printf '%s%s%s  %s  %s  %s\n' "$C_BAR" "$S_START" "$C_OFF" "$C_PILL" "$*" "$C_OFF"; bar; }
outro() { printf '%s%s%s  %s\n' "$C_BAR" "$S_END" "$C_OFF" "$*"; }
log_info() { printf '%s%s%s  %s\n' "$C_BLUE" "$S_INFO" "$C_OFF" "$1"; shift; for l in "$@"; do dimline "$l"; done; bar; }
log_ok() { printf '%s%s%s  %s\n' "$C_GREEN" "$S_SUBMIT" "$C_OFF" "$1"; shift; for l in "$@"; do dimline "$l"; done; bar; }
log_warn() { printf '%s%s%s  %s\n' "$C_YELLOW" "$S_WARN" "$C_OFF" "$1"; shift; for l in "$@"; do dimline "$l"; done; bar; }
log_err() { printf '%s%s%s  %s\n' "$C_RED" "$S_ERR" "$C_OFF" "$1"; shift; for l in "$@"; do dimline "$l"; done; bar; }
fail() { # fail MESSAGE [detail...]  -> prints the error and leaves
    log_err "$@"
    outro "${C_RED}Nothing was changed.${C_OFF}"
    exit 1
}

# note TITLE BODY   (BODY: one "X|text" per line; X = + ok, - missing, ! warning, space = plain)
note() {
    title=$1
    body=$2
    w=$((${#title} + 1))
    oldifs=$IFS
    IFS='
'
    set -f
    for l in $body; do
        t=${l#?|}
        [ $((${#t} + 2)) -gt "$w" ] && w=$((${#t} + 2))
    done
    printf '%s%s%s  %s%s%s %s%s%s%s\n' "$C_GREEN" "$S_SUBMIT" "$C_OFF" "$C_BOLD" "$title" "$C_OFF" \
        "$C_BAR" "$(rep $((w + 1 - ${#title})) "$S_DASH")" "$S_TR" "$C_OFF"
    printf '%s%s%s%s%s%s\n' "$C_BAR" "$S_BAR" "$(spaces $((w + 4)))" "$S_BAR" "$C_OFF" ''
    for l in $body; do
        s=${l%%|*}
        t=${l#?|}
        case $s in
            +) icon="${C_GREEN}${S_OK}${C_OFF}" ;;
            -) icon="${C_RED}${S_BAD}${C_OFF}" ;;
            '!') icon="${C_YELLOW}${S_WARN}${C_OFF}" ;;
            *) icon=' ' ;;
        esac
        printf '%s%s%s  %s %s%s  %s%s%s\n' "$C_BAR" "$S_BAR" "$C_OFF" "$icon" "$t" \
            "$(spaces $((w - ${#t} - 2)))" "$C_BAR" "$S_BAR" "$C_OFF"
    done
    set +f
    IFS=$oldifs
    printf '%s%s%s%s%s%s\n' "$C_BAR" "$S_BAR" "$(spaces $((w + 4)))" "$S_BAR" "$C_OFF" ''
    printf '%s%s%s%s%s\n' "$C_BAR" "$S_BL" "$(rep $((w + 4)) "$S_DASH")" "$S_BR" "$C_OFF"
    bar
}

# ---------------------------------------------------------------- terminal handling
SAVED_STTY=
SPIN_PID=
cleanup() {
    [ -n "$SPIN_PID" ] && kill "$SPIN_PID" 2>/dev/null
    [ -n "$SAVED_STTY" ] && stty "$SAVED_STTY" 2>/dev/null
    SAVED_STTY=
    [ -t 1 ] && printf '%s[?25h' "$ESC"
    [ -n "${TMP_TARGET:-}" ] && rm -rf "$TMP_TARGET"
    [ -n "${LOG:-}" ] && rm -f "$LOG"
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM HUP

getkey() { # sets KEY: up, down, enter, esc, or the typed character
    k=$(dd bs=1 count=1 2>/dev/null) || k=
    case $k in
        "$ESC")
            stty min 0 time 2 2>/dev/null
            rest=$(dd bs=1 count=2 2>/dev/null)
            stty min 1 time 0 2>/dev/null
            case $rest in
                '[A' | 'OA') KEY=up ;;
                '[B' | 'OB') KEY=down ;;
                '[C' | 'OC') KEY=right ;;
                '[D' | 'OD') KEY=left ;;
                *) KEY=esc ;;
            esac
            ;;
        '' | ' ') KEY=enter ;;
        *) KEY=$k ;;
    esac
}

# choose TITLE "Label|hint" ...   -> CHOICE = number picked (1-based). Start on $CHOICE_DEFAULT.
CHOICE_DEFAULT=1
CHOICE=1
choose() {
    title=$1
    shift
    n=$#
    sel=$CHOICE_DEFAULT
    maxl=0
    for o in "$@"; do
        lab=${o%%|*}
        [ ${#lab} -gt "$maxl" ] && maxl=${#lab}
    done
    if [ ! -t 0 ] || [ ! -t 1 ]; then # no keyboard: plain numbered question, answer on stdin
        printf '%s%s%s  %s\n' "$C_CYAN" "$S_ACTIVE" "$C_OFF" "$title"
        i=1
        for o in "$@"; do
            line "$i) ${o%%|*}"
            i=$((i + 1))
        done
        printf '%s%s%s  Choose [1-%s, Enter = %s]: ' "$C_BAR" "$S_END" "$C_OFF" "$n" "$sel"
        read -r ans || ans=
        printf '\n'
        pick=$sel
        case $ans in
            '') ;;
            *[!0-9]*)
                i=1
                for o in "$@"; do
                    lab=${o%%|*}
                    first=$(printf '%s' "$lab" | cut -c1 | tr 'A-Z' 'a-z')
                    a1=$(printf '%s' "$ans" | cut -c1 | tr 'A-Z' 'a-z')
                    [ "$first" = "$a1" ] && pick=$i && break
                    i=$((i + 1))
                done
                ;;
            *) [ "$ans" -ge 1 ] && [ "$ans" -le "$n" ] && pick=$ans ;;
        esac
        CHOICE=$pick
        i=1
        for o in "$@"; do
            [ "$i" = "$pick" ] && lab=${o%%|*}
            i=$((i + 1))
        done
        log_ok "$title" "$lab"
        return 0
    fi

    SAVED_STTY=$(stty -g 2>/dev/null)
    stty -icanon -echo min 1 time 0 2>/dev/null
    printf '%s[?25l' "$ESC"
    first=1
    while :; do
        [ "$first" = 0 ] && printf '%s[%sA%s[J' "$ESC" $((n + 2)) "$ESC"
        first=0
        printf '%s%s%s  %s\n' "$C_CYAN" "$S_ACTIVE" "$C_OFF" "$title"
        i=1
        for o in "$@"; do
            lab=${o%%|*}
            hint=${o#*|}
            [ "$hint" = "$o" ] && hint=
            pad=$(spaces $((maxl - ${#lab})))
            if [ "$i" = "$sel" ]; then
                printf '%s%s%s  %s%s%s %s%s%s  %s%s%s%s\n' "$C_CYAN" "$S_BAR" "$C_OFF" "$C_GREEN" "$S_RON" "$C_OFF" \
                    "$C_BOLD" "$i  $lab" "$C_OFF" "$pad" "$C_DIM" "$hint" "$C_OFF"
            else
                printf '%s%s%s  %s%s%s %s%s  %s%s%s\n' "$C_CYAN" "$S_BAR" "$C_OFF" "$C_DIM" "$S_ROFF" "$C_OFF" \
                    "$C_DIM" "$i  $lab$pad" "$hint" "$C_OFF"
            fi
            i=$((i + 1))
        done
        printf '%s%s%s  %s%s%s\n' "$C_CYAN" "$S_END" "$C_OFF" "$C_DIM" "up/down move   1-$n choose   enter confirm   q quit" "$C_OFF"
        getkey
        case $KEY in
            up) sel=$((sel - 1)); [ "$sel" -lt 1 ] && sel=$n ;;
            down) sel=$((sel + 1)); [ "$sel" -gt "$n" ] && sel=1 ;;
            enter) break ;;
            q | Q | esc) sel=0; break ;;
            [1-9]) [ "$KEY" -le "$n" ] && { sel=$KEY; break; } ;;
        esac
    done
    stty "$SAVED_STTY" 2>/dev/null
    SAVED_STTY=
    printf '%s[?25h' "$ESC"
    printf '%s[%sA%s[J' "$ESC" $((n + 2)) "$ESC"
    if [ "$sel" = 0 ]; then
        printf '%s%s%s  %s\n' "$C_RED" "$S_ERR" "$C_OFF" "$title"
        dimline "cancelled"
        bar
        CHOICE=0
        return 1
    fi
    i=1
    for o in "$@"; do
        [ "$i" = "$sel" ] && lab=${o%%|*}
        i=$((i + 1))
    done
    log_ok "$title" "$lab"
    CHOICE=$sel
    return 0
}

# confirm QUESTION [default: yes|no]   -> returns 0 for yes
confirm() {
    CHOICE_DEFAULT=1
    [ "${2:-yes}" = no ] && CHOICE_DEFAULT=2
    choose "$1" "Yes" "No" || { CHOICE_DEFAULT=1; return 1; }
    CHOICE_DEFAULT=1
    [ "$CHOICE" = 1 ]
}

# spin "message" command args...   -> runs the command quietly with a spinner, output kept in $LOG
LOG=
spin() {
    msg=$1
    shift
    LOG=$(mktemp "${TMPDIR:-/tmp}/nysakura-install.XXXXXX")
    if [ ! -t 1 ]; then
        printf '%s  %s ...\n' "$S_INFO" "$msg"
        "$@" >"$LOG" 2>&1
        rc=$?
    else
        printf '%s[?25l' "$ESC"
        "$@" >"$LOG" 2>&1 &
        SPIN_PID=$!
        while kill -0 "$SPIN_PID" 2>/dev/null; do
            for f in $SPIN_FRAMES; do
                printf '\r%s[K%s%s%s  %s' "$ESC" "$C_CYAN" "$f" "$C_OFF" "$msg"
                sleep 0.1 2>/dev/null || sleep 1
            done
        done
        wait "$SPIN_PID"
        rc=$?
        SPIN_PID=
        printf '\r%s[K%s[?25h' "$ESC" "$ESC"
    fi
    if [ "$rc" = 0 ]; then
        log_ok "$msg"
    else
        log_err "$msg failed" "The last lines of the log:"
        tail -n 12 "$LOG" | cut -c1-100 | while IFS= read -r l; do dimline "  $l"; done
        bar
    fi
    rm -f "$LOG"
    LOG=
    return "$rc"
}

have() { command -v "$1" >/dev/null 2>&1; }

# ---------------------------------------------------------------- the system
if [ -r /etc/os-release ]; then
    DISTRO=$(. /etc/os-release 2>/dev/null && printf '%s' "${PRETTY_NAME:-${NAME:-Linux}}")
else
    DISTRO=Linux
fi
ARCH=$(uname -m 2>/dev/null || echo unknown)

PM=
for p in pacman apt-get dnf zypper apk xbps-install emerge; do
    if have "$p"; then
        PM=$p
        break
    fi
done

BUILD_USER=${SUDO_USER:-${DOAS_USER:-}}
[ "$BUILD_USER" = root ] && BUILD_USER=
if [ -n "$BUILD_USER" ] && ! id "$BUILD_USER" >/dev/null 2>&1; then BUILD_USER=; fi
BUILD_AS_USER=0
if [ -n "$BUILD_USER" ]; then
    BUILD_HOME=$(getent passwd "$BUILD_USER" 2>/dev/null | cut -d: -f6)
    [ -n "$BUILD_HOME" ] || BUILD_HOME=$(eval "printf '%s' ~$BUILD_USER")
    if have runuser || have sudo; then BUILD_AS_USER=1; fi
else
    BUILD_HOME=${HOME:-/root}
fi
USER_PATH="$BUILD_HOME/.cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin:${PATH:-}"

# run a command as the person who started sudo (so the build never leaves root-owned files behind)
as_user() {
    if [ "$BUILD_AS_USER" = 1 ]; then
        if have runuser; then
            runuser -u "$BUILD_USER" -- env "HOME=$BUILD_HOME" "PATH=$USER_PATH" "$@"
        else
            sudo -u "$BUILD_USER" -H env "PATH=$USER_PATH" "$@"
        fi
    else
        env "PATH=$USER_PATH" "$@"
    fi
}

VERSION=$(sed -n 's/^version *= *"\(.*\)"/\1/p' "$HERE/Cargo.toml" 2>/dev/null | head -n 1)
[ -n "$VERSION" ] || VERSION=unknown

installed_version() {
    if [ -x "$BIN/$NAME" ]; then "$BIN/$NAME" --version 2>/dev/null; fi
}

# ---------------------------------------------------------------- dependencies
RUST_OK=0 CARGO_OK=0 CC_OK=0 GIT_OK=0
RUST_TXT= CARGO_TXT= CC_TXT= GIT_TXT=

check_deps() {
    RUST_OK=0 CARGO_OK=0 CC_OK=0 GIT_OK=0
    rv=$(as_user rustc --version 2>/dev/null)
    if [ -n "$rv" ]; then
        set -- $rv
        ver=${2:-0.0.0}
        ver=${ver%%-*}
        major=${ver%%.*}
        minor=${ver#*.}
        minor=${minor%%.*}
        RUST_TXT="rustc $ver (needs 1.$MIN_RUST_MINOR or newer)"
        if [ "$major" -gt 1 ] 2>/dev/null || [ "$minor" -ge "$MIN_RUST_MINOR" ] 2>/dev/null; then RUST_OK=1; fi
    else
        RUST_TXT="rustc not found (needs 1.$MIN_RUST_MINOR or newer)"
    fi
    cv=$(as_user cargo --version 2>/dev/null)
    if [ -n "$cv" ]; then
        CARGO_OK=1
        CARGO_TXT=$cv
    else
        CARGO_TXT="cargo not found"
    fi
    ccp=$(as_user sh -c 'command -v cc || command -v gcc || command -v clang' 2>/dev/null | head -n 1)
    if [ -n "$ccp" ]; then
        CC_OK=1
        CC_TXT="C linker: $ccp"
    else
        CC_TXT="C compiler/linker not found (cc, gcc or clang)"
    fi
    gv=$(git --version 2>/dev/null)
    if [ -n "$gv" ]; then
        GIT_OK=1
        GIT_TXT="$gv (used by Update)"
    else
        GIT_TXT="git not found (only needed for Update)"
    fi
}

dep_note() {
    b=
    nl='
'
    row() { b="$b$1|$2$nl"; }
    [ "$RUST_OK" = 1 ] && row + "$RUST_TXT" || row - "$RUST_TXT"
    [ "$CARGO_OK" = 1 ] && row + "$CARGO_TXT" || row - "$CARGO_TXT"
    [ "$CC_OK" = 1 ] && row + "$CC_TXT" || row - "$CC_TXT"
    [ "$GIT_OK" = 1 ] && row + "$GIT_TXT" || row '!' "$GIT_TXT"
    note "Dependencies" "${b%"$nl"}"
}

pkgs_for() { # pkgs_for rust|cc|git
    case "$PM:$1" in
        pacman:rust) echo "rust" ;;
        pacman:cc) echo "gcc" ;;
        apt-get:rust) echo "cargo rustc" ;;
        apt-get:cc) echo "gcc" ;;
        dnf:rust | zypper:rust | xbps-install:rust | apk:rust) echo "cargo rust" ;;
        dnf:cc | zypper:cc | xbps-install:cc) echo "gcc" ;;
        apk:cc) echo "gcc musl-dev" ;;
        emerge:rust) echo "dev-lang/rust-bin" ;;
        emerge:cc) echo "sys-devel/gcc" ;;
        *:git) [ "$PM" = emerge ] && echo "dev-vcs/git" || echo "git" ;;
    esac
}

pm_install() { # pm_install pkgs...
    case $PM in
        pacman) pacman -S --needed --noconfirm "$@" ;;
        apt-get) sh -c 'apt-get update && apt-get install -y "$@"' sh "$@" ;;
        dnf) dnf install -y "$@" ;;
        zypper) zypper --non-interactive install "$@" ;;
        apk) apk add "$@" ;;
        xbps-install) xbps-install -Sy "$@" ;;
        emerge) emerge --ask=n "$@" ;;
        *) return 1 ;;
    esac
}

# ensure_deps [needgit]  -> makes sure rust, cargo and a C linker exist (offers to install them)
ensure_deps() {
    needgit=${1:-0}
    check_deps
    dep_note
    missing=
    if [ "$RUST_OK" = 0 ] || [ "$CARGO_OK" = 0 ]; then missing="$missing $(pkgs_for rust)"; fi
    [ "$CC_OK" = 0 ] && missing="$missing $(pkgs_for cc)"
    if [ "$needgit" = 1 ] && [ "$GIT_OK" = 0 ]; then missing="$missing $(pkgs_for git)"; fi
    missing=${missing# }
    [ -z "${missing% }" ] && return 0

    if { [ "$RUST_OK" = 0 ] || [ "$CARGO_OK" = 0 ]; } && have rustup; then
        fail "rustup is installed but has no Rust toolchain yet" "Run this as your normal user, then start the installer again:" "  rustup default stable"
    fi
    if [ -z "$PM" ]; then
        fail "Your package manager was not recognised" "Install these yourself, then run the installer again:" \
            "  Rust 1.$MIN_RUST_MINOR+ (rustc and cargo), a C compiler, git" "  Rust from https://rustup.rs works everywhere"
    fi
    log_warn "Missing: $missing" "These are installed with $PM."
    if ! confirm "Install the missing packages now?"; then
        fail "Cannot continue without them" "Install: $missing"
    fi
    # shellcheck disable=SC2086
    spin "Installing $missing" pm_install $missing || fail "The package manager failed" "Fix the problem above and run the installer again."
    check_deps
    if [ "$RUST_OK" = 0 ] || [ "$CARGO_OK" = 0 ] || [ "$CC_OK" = 0 ]; then
        fail "Still missing after the install" "$RUST_TXT" "$CC_TXT" "Your distribution may ship an old Rust. Get a current one from https://rustup.rs"
    fi
}

# ---------------------------------------------------------------- build and copy
TARGET=
TMP_TARGET=
build() {
    if [ "$BUILD_AS_USER" = 1 ]; then
        TARGET=$HERE/target
    else
        TMP_TARGET=$(mktemp -d "${TMPDIR:-/tmp}/nysakura-build.XXXXXX")
        TARGET=$TMP_TARGET
    fi
    spin "Compiling nysakura $VERSION (release)" \
        as_user env "CARGO_TARGET_DIR=$TARGET" sh -c 'cd "$1" && exec cargo build --release' sh "$HERE" ||
        fail "The build failed" "Fix the problem above, or open an issue with the log."
    [ -x "$TARGET/release/$NAME" ] || fail "The build finished but no program was produced"
}

install_files() {
    put() { # put MODE SRC DEST
        [ -f "$2" ] || return 0
        install -Dm"$1" "$2" "$3"
    }
    put 755 "$TARGET/release/$NAME" "$BIN/$NAME" &&
        put 644 "$HERE/man/$NAME.1" "$MAN/$NAME.1" &&
        put 644 "$HERE/completions/$NAME.bash" "$BASH_DIR/$NAME" &&
        put 644 "$HERE/completions/_$NAME" "$ZSH_DIR/_$NAME" &&
        put 644 "$HERE/completions/$NAME.fish" "$FISH_DIR/$NAME.fish" &&
        put 644 "$HERE/README.md" "$DOC_DIR/README.md" &&
        put 644 "$HERE/LICENSE" "$LIC_DIR/LICENSE"
}

# the old Python version was called xsakura
legacy_files() {
    for d in "$PREFIX" "$BUILD_HOME/.local"; do
        for f in "$d/bin/xsakura" "$d/share/man/man1/xsakura.1" "$d/share/bash-completion/completions/xsakura" \
            "$d/share/zsh/site-functions/_xsakura" "$d/share/fish/vendor_completions.d/xsakura.fish" \
            "$BUILD_HOME/.config/fish/completions/xsakura.fish"; do
            [ -e "$f" ] && printf '%s\n' "$f"
        done
    done
}

handle_legacy() {
    old=$(legacy_files | sort -u)
    [ -n "$old" ] || return 0
    nl='
'
    b=
    for f in $old; do b="$b |$f$nl"; done
    note "Old xsakura found" "${b%"$nl"}"
    log_warn "nysakura replaces the old Python tool xsakura" "Both would work side by side, but the old one is no longer updated."
    if confirm "Remove the old xsakura files?"; then
        for f in $old; do rm -f "$f"; done
        log_ok "Removed the old xsakura"
    else
        log_info "Left the old xsakura where it is"
    fi
}

check_pacman_xsakura() {
    if [ -e /usr/bin/xsakura ]; then
        log_warn "xsakura is also installed by your package manager" "Remove it with:  sudo pacman -Rns xsakura   (or: yay -Rns xsakura)"
    fi
}

path_hint() {
    case ":${PATH:-}:" in
        *":$BIN:"*) ;;
        *) log_warn "$BIN is not in your PATH" "Add it to your shell config, then open a new terminal." ;;
    esac
}

summary() { # summary headline newver
    nl='
'
    note "$1" " |Version      $2$nl |Program      $BIN/$NAME$nl |Manual       man nysakura$nl |Completions  bash, zsh and fish"
    log_info "Try it" "  nysakura            a tall sakura tree" "  nysakura -mini      a shorter, wider tree" "  nysakura -blue      any color, see nysakura --list"
}

# ---------------------------------------------------------------- actions
do_install() {
    [ -f "$HERE/Cargo.toml" ] && [ -f "$HERE/src/main.rs" ] ||
        fail "Cannot find the nysakura source" "Run the installer from inside the nysakura folder you cloned or unzipped."
    ensure_deps 0
    handle_legacy
    build
    spin "Copying files to $PREFIX" install_files || fail "Could not copy the files to $PREFIX"
    newv=$("$BIN/$NAME" --version 2>/dev/null) || fail "The installed program did not start" "Run:  $BIN/$NAME --version"
    check_pacman_xsakura
    path_hint
    summary "Installed" "${newv#"$NAME "}"
    outro "${C_GREEN}Done!${C_OFF} Type: ${C_BOLD}nysakura${C_OFF}"
}

do_update() {
    oldv=$(installed_version)
    if [ -z "$oldv" ]; then
        log_warn "nysakura is not installed yet" "I will do a fresh install instead."
    fi
    [ -f "$HERE/Cargo.toml" ] && [ -f "$HERE/src/main.rs" ] ||
        fail "Cannot find the nysakura source" "Run the installer from inside the nysakura folder you cloned."
    ensure_deps 1
    root=$(as_user git -C "$HERE" rev-parse --show-toplevel 2>/dev/null)
    if [ "$GIT_OK" = 1 ] && [ -n "$root" ] && [ "$(cd "$root" && pwd -P)" = "$HERE" ]; then
        spin "Downloading the newest version from GitHub" as_user git -C "$HERE" pull --ff-only ||
            fail "git pull failed" "If you edited files in this folder, run 'git status' in it, or clone the repository again:" "  git clone https://github.com/77nyan/nysakura.git"
    else
        log_warn "This folder is not a git clone, so it cannot download updates" \
            "Updating from the files in this folder. For the newest version:" "  git clone https://github.com/77nyan/nysakura.git"
    fi
    VERSION=$(sed -n 's/^version *= *"\(.*\)"/\1/p' "$HERE/Cargo.toml" | head -n 1)
    handle_legacy
    build
    spin "Copying files to $PREFIX" install_files || fail "Could not copy the files to $PREFIX"
    newv=$("$BIN/$NAME" --version 2>/dev/null) || fail "The installed program did not start" "Run:  $BIN/$NAME --version"
    path_hint
    if [ -z "$oldv" ]; then
        summary "Installed" "${newv#"$NAME "}"
        outro "${C_GREEN}Done!${C_OFF} Type: ${C_BOLD}nysakura${C_OFF}"
    elif [ "$oldv" = "$newv" ]; then
        summary "Up to date" "${newv#"$NAME "}"
        outro "${C_GREEN}You are up to date:${C_OFF} $newv"
    else
        summary "Updated" "${newv#"$NAME "}"
        outro "${C_GREEN}Updated:${C_OFF} $oldv  ->  $newv"
    fi
}

do_uninstall() {
    nl='
'
    found=
    for f in "$BIN/$NAME" "$MAN/$NAME.1" "$BASH_DIR/$NAME" "$ZSH_DIR/_$NAME" "$FISH_DIR/$NAME.fish" \
        "$DOC_DIR/README.md" "$LIC_DIR/LICENSE"; do
        [ -e "$f" ] && found="$found |$f$nl"
    done
    if [ -z "$found" ]; then
        log_warn "nysakura is not installed in $PREFIX" "If you installed it somewhere else, remove the files there by hand."
        handle_legacy
        outro "Nothing to remove."
        exit 0
    fi
    note "These files will be removed" "${found%"$nl"}"
    if ! confirm "Remove nysakura from $PREFIX?" no; then
        outro "Nothing was changed."
        exit 0
    fi
    for f in "$BIN/$NAME" "$MAN/$NAME.1" "$BASH_DIR/$NAME" "$ZSH_DIR/_$NAME" "$FISH_DIR/$NAME.fish" \
        "$DOC_DIR/README.md" "$LIC_DIR/LICENSE"; do
        rm -f "$f"
    done
    rmdir "$DOC_DIR" "$LIC_DIR" 2>/dev/null
    log_ok "nysakura was removed"
    handle_legacy
    log_info "Your source folder was left alone" "Delete it yourself if you do not need it:  $HERE"
    outro "${C_GREEN}Done.${C_OFF} Goodbye!"
}

# ---------------------------------------------------------------- main
if [ $# -gt 0 ]; then
    case $1 in
        -h | --help)
            intro "nysakura installer"
            log_info "Just run it, there are no options" "  sudo ./install.sh"
            outro "A menu lets you Install, Update or Uninstall."
            exit 0
            ;;
        *)
            intro "nysakura installer"
            fail "This installer takes no options: $1" "Run it like this:  sudo ./install.sh"
            ;;
    esac
fi

intro "nysakura installer"

if [ "$(id -u)" != 0 ]; then
    fail "This installer needs root" "Run it like this:" "  sudo ./install.sh" "If it says 'permission denied', first run:  chmod +x install.sh"
fi

log_info "Welcome to the nysakura installer" "nysakura $VERSION: a cherry blossom tree that grows in your terminal" \
    "$DISTRO, $ARCH${PM:+, package manager: $PM}"

cur=$(installed_version)
if [ -n "$cur" ]; then
    CHOICE_DEFAULT=2
    log_info "Installed: $cur" "$BIN/$NAME"
else
    log_info "nysakura is not installed yet"
fi

choose "What would you like to do?" \
    "Install|build nysakura and install it to $PREFIX" \
    "Update|get the newest version and reinstall" \
    "Uninstall|remove nysakura from $PREFIX" \
    "Quit|leave without changing anything" || {
    outro "Nothing was changed."
    exit 0
}
CHOICE_DEFAULT=1

case $CHOICE in
    1) do_install ;;
    2) do_update ;;
    3) do_uninstall ;;
    *) outro "Nothing was changed." ;;
esac
