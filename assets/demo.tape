# Regenerate with: vhs assets/demo.tape (from the repository root)

Output assets/demo.gif

Set Shell "bash"
Set FontSize 20
Set Width 1440
Set Height 1000
Set Padding 24
Set Framerate 12
Set Theme "Catppuccin Mocha"
Set TypingSpeed 60ms

Hide
Type `cargo build --release --quiet && export PATH="$PWD/target/release:$PATH" PS1='\[\e[1;35m\]$ \[\e[0m\]' && clear`
Enter
Wait@300s /^\$\s*$/
Show

Type "poker-rs"
Sleep 400ms
Enter
Sleep 2s

# Menu: bump the number of players, then start the table
Down
Sleep 300ms
Up
Sleep 300ms
Type "+"
Sleep 800ms
Enter
Sleep 1s

# Deal a hand and play it by calling / checking
Space
Sleep 2.5s
Type "c"
Sleep 2.5s
Type "c"
Sleep 2.5s
Type "c"
Sleep 2.5s
Type "c"
Sleep 2.5s
Type "c"
Sleep 2.5s
Type "c"
Sleep 2.5s
Type "c"
Sleep 2.5s
Type "c"
Sleep 3s

# Hand history and in-game help
Type "h"
Sleep 3s
Escape
Sleep 500ms
Type "?"
Sleep 3s
Escape
Sleep 500ms

Type "m"
Sleep 800ms
Type "q"
Sleep 500ms
