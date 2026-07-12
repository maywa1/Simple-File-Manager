# SFM - Simple File Manager

A minimal TUI file manager built in Rust that feels like using a shell. Type to search, press Enter to act.

# Demo Video

https://github.com/user-attachments/assets/b28ca122-2f0f-4d67-b9da-212ad6ce527d

## Features

- **Fuzzy search** for files and directories using [nucleo](https://github.com/lotabout/nucleo)
- **Shell-like navigation** -- type `..` to go up, `<dir>/` to enter a directory
- **Tab autocompletion** -- completes the common prefix among matches, just like a shell
- **Glob patterns** -- type `*.pdf` to select all matching files, then act on them in bulk
- **File operations** -- open, rename, move, delete, copy path to clipboard
- **Create files/directories** -- type a non-existent path and press Enter

## Installation

### From source

```sh
cargo install --path .
```

### Build manually

```sh
git clone https://github.com/maywai/sfm.git
cd sfm
cargo build --release
```

The binary will be at `target/release/sfm`.

## Usage

SFM starts in your current working directory. Type in the search bar to filter files and directories in real time.

### Navigation

| Input          | Action                              |
| -------------- | ----------------------------------- |
| `..`           | Go to parent directory              |
| `<dir_name>/`  | Enter `dir_name` if it exists       |
| `Tab`          | Autocomplete (common prefix)        |
| `Enter`        | Select entry / open action menu     |
| `Ctrl+C`       | Quit                                |

### Single file actions

Press `Enter` on a file or directory to open the action menu:

| Key   | Action            |
| ----- | ----------------- |
| `o`   | Open with xdg-open |
| `e`   | Open in $EDITOR   |
| `r`   | Rename            |
| `d`   | Delete (confirm)  |
| `y`   | Copy path         |
| `m`   | Move              |

### Bulk actions

Type a glob pattern (e.g. `*.txt`) and press `Enter` to select all matching files:

| Key   | Action          |
| ----- | --------------- |
| `d`   | Delete all      |
| `y`   | Copy all paths  |
| `m`   | Move all        |

### Input shortcuts

| Key         | Action                              |
| ----------- | ----------------------------------- |
| `Ctrl+W`    | Delete previous word                |
| `Backspace` | Delete character                    |
| `Left/Right`| Move cursor                         |
| `Esc`       | Cancel / go back                    |

### Creating files or directories

Type a name that doesn't exist and press `Enter`, then choose:

| Key   | Action            |
| ----- | ----------------- |
| `d`   | Create directory  |
| `f`   | Create file       |

## Dependencies

| Crate     | Purpose                |
| --------- | ---------------------- |
| ratatui   | Terminal UI framework  |
| nucleo    | Fuzzy matching engine  |
| arboard   | Clipboard access       |
| walkdir   | Recursive dir traversal |
| color-eyre| Error reporting        |

## License

[GPLv3](LICENSE)
