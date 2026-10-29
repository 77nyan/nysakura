# bash completion for nysakura
_nysakura() {
    local cur prev
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"
    case "$prev" in
        -s|--seed|-t|--time|--cycle|--speed|--grow|--petals|--wind|--scale|--width|--height|--fps|-hex|--hex)
            return ;;
    esac
    local colors="-sakura -pink -red -orange -yellow -gold -green -cyan -blue -purple -magenta -white -rainbow -hex -mini"
    local opts="-s --seed -p --print -i --instant -t --time --cycle --speed --grow --petals --wind --scale --width --height --fps --truecolor --256 -l --list -h --help -v --version"
    COMPREPLY=( $(compgen -W "$colors $opts" -- "$cur") )
}
complete -F _nysakura nysakura
