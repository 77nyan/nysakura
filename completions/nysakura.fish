# fish completion for nysakura
complete -c nysakura -f
for c in sakura pink red orange yellow gold green cyan blue purple magenta white rainbow
    complete -c nysakura -o $c -d "$c tree"
end
complete -c nysakura -o hex -x -d 'custom color (RRGGBB)'
complete -c nysakura -o mini -d 'shorter, wider tree'
complete -c nysakura -s s -l seed -x -d 'reproducible tree'
complete -c nysakura -s p -l print -d 'print the finished tree and exit'
complete -c nysakura -s i -l instant -d 'skip the growing animation'
complete -c nysakura -s t -l time -x -d 'quit after N seconds'
complete -c nysakura -l cycle -x -d 'grow a new tree N seconds after the last one'
complete -c nysakura -l speed -x -d 'animation speed multiplier'
complete -c nysakura -l grow -x -d 'seconds the tree needs to grow'
complete -c nysakura -l petals -x -d 'number of falling petals'
complete -c nysakura -l wind -x -d 'wind strength, negative blows left'
complete -c nysakura -l scale -x -d 'tree size multiplier'
complete -c nysakura -l width -x -d 'width in cells'
complete -c nysakura -l height -x -d 'height in cells'
complete -c nysakura -l fps -x -d 'frames per second'
complete -c nysakura -l truecolor -d 'force 24-bit color'
complete -c nysakura -l 256 -d 'force 256-color mode'
complete -c nysakura -s l -l list -d 'list the colors'
complete -c nysakura -s h -l help -d 'show help'
complete -c nysakura -s v -l version -d 'show version'
