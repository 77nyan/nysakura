# fish completion for xsakura
complete -c xsakura -f
for c in sakura pink red orange yellow gold green cyan blue purple magenta white rainbow
    complete -c xsakura -o $c -d "$c tree"
end
complete -c xsakura -o hex -x -d 'custom color (RRGGBB)'
complete -c xsakura -o mini -d 'shorter, wider tree'
complete -c xsakura -s s -l seed -x -d 'reproducible tree'
complete -c xsakura -s p -l print -d 'print the finished tree and exit'
complete -c xsakura -s i -l instant -d 'skip the growing animation'
complete -c xsakura -s t -l time -x -d 'quit after N seconds'
complete -c xsakura -l cycle -x -d 'grow a new tree N seconds after the last one'
complete -c xsakura -l speed -x -d 'animation speed multiplier'
complete -c xsakura -l grow -x -d 'seconds the tree needs to grow'
complete -c xsakura -l petals -x -d 'number of falling petals'
complete -c xsakura -l wind -x -d 'wind strength, negative blows left'
complete -c xsakura -l scale -x -d 'tree size multiplier'
complete -c xsakura -l width -x -d 'width in cells'
complete -c xsakura -l height -x -d 'height in cells'
complete -c xsakura -l fps -x -d 'frames per second'
complete -c xsakura -l truecolor -d 'force 24-bit color'
complete -c xsakura -l 256 -d 'force 256-color mode'
complete -c xsakura -s l -l list -d 'list the colors'
complete -c xsakura -s h -l help -d 'show help'
complete -c xsakura -s v -l version -d 'show version'
