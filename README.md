# Usage
Typing in search bar:
- ".." - moves to previous directory;
- "\<dir_name\>/" - moves to dir_name if it exists
Keybinds:
- Press `Tab` to autocomplete with the file/directory that starts with the substring which was typed, if there is more then one option, it will complete with the common substring between all files that share the same substring from the user input (similar to pressing tab on a shell)
- Press `Enter` to do an action with the typed file/directory, if nothing is typed the current directory is used
Available actions:
- `y` - copy path
- `r` - copy path
- `m` - move
- `d` - delete
- `o` - open using xdg-open
