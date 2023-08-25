# Watchbind

[![Build status](https://github.com/fritzrehde/watchbind/actions/workflows/ci.yml/badge.svg)](https://github.com/fritzrehde/watchbind/actions)
[![Releases](https://img.shields.io/github/v/release/fritzrehde/watchbind?logo=GitHub)](https://github.com/fritzrehde/watchbind/releases)
[![Crates.io](https://img.shields.io/crates/v/watchbind?logo=Rust)](https://crates.io/crates/watchbind)

*Turn any shell command into a powerful TUI with custom keybindings.*

![screenshot](https://raw.githubusercontent.com/fritzrehde/i/master/watchbind/screenshot-light.png#gh-light-mode-only)
![screenshot](https://raw.githubusercontent.com/fritzrehde/i/master/watchbind/screenshot-dark.png#gh-dark-mode-only)

## Table of Contents

- [Features](#features)
- [Installation](#installation)
- [Customizations](#customizations)
  - [Keybindings](#customizations)
  - [Formatting with Field Separators and Selections](#formatting-with-field-separators-and-selections)
  - [Styling](#styling)
- [Tips](#tips)

## Features

- **Customizability**: All keybindings and styles (colors and boldness) are customizable.
- **Flexibility**: You can specify settings using CLI options, a TOML config file, or both.
- **Speed**: Written in asynchronous Rust with [Tokio](https://tokio.rs/).

## Installation

### From binaries

The [releases page](https://github.com/fritzrehde/watchbind/releases) contains pre-compiled binaries for Linux, macOS and Windows.

### From [crates.io](https://crates.io/crates/watchbind)

```shell
cargo install watchbind
```

## How it works

Watchbind is a command-line tool that aims to help you build custom TUIs from static CLI commands very easily.
It works by specifying a "watched command" that outputs some lines to stdout that you want to observe.
Then, we make this static output **dynamic** by re-executing it at a specified watch rate, and we make the TUI **interactive** through custom keybindings that can operate on the individual output lines.


## Customizations

There are several ways to customize the settings:
1. A TOML config file, specified with `watchbind --config-file <FILE>`, overrides all default settings ([examples/](examples/)).
2. The command-line options override all other settings (i.e. all TOML and default settings).

All ways of configuring `watchbind` (TOML and CLI options) can be used at the same time, and `watchbind` will automatically figure out which settings to use according to the above hierarchy.

Personally, I recommend using the CLI options for small one liners and a TOML config file for more complex scripts.

### Keybindings

#### Via Command-Line Arguments

On the command line, you can specify a comma-separated list of keybindings, where each keybinding is in the format `KEY:OP[+OP]*`.
One `KEY` can be bound to multiple operations, therefore, the syntax for each list of operations (`OPS`) is `OP[+OP]*`.
The operations are separated by `+` and executed in succession (one after the other).

**TLDR**:
- Individual keybindings are separated by `,`
- A keybinding is a pair of key and (multiple) operations separated by `:`
- Multiple operations are separated by `+`

#### Via TOML Config File

In a TOML config file, specify keybindings like so:
```toml
[keybindings]
"KEY" = [ "OP" ]
"KEY" = [ "OP", "OP", "OP" ]
"KEY" = [ 
  "OP",
  "OP"
]
```

This syntax differs from the command-line syntax because using the TOML array feature is more expressive and more native to the TOML file format.
Furthermore, this allows you to use the `+` character in your commands.
It also doesn't require escaping shell specific characters like `$` in  (read more [in this section](#subshell)).

You can find some keybinding examples in the [`examples/`](examples/) directory.

#### Keys

All supported `KEY` values:
```
<MODIFIER>+<CODE>
<CODE>
```

All supported `MODIFIER` values:
```
alt
ctrl
```

All supported `CODE` values:
```
esc
enter
left
right
up
down
home
end
pageup
pagedown
backtab
backspace
del
delete
insert
ins
f1
f2
f3
f4
f5
f6
f7
f8
f9
f10
f11
f12
space
tab
<any single character>
```

#### Operations

All supported `OP` values:

Operation | Action
:-- | :--
exit | Quit watchbind
reload | Reload the watched command manually, resets interval timer
cursor \[down\|up\] \<N\> | Move cursor \[down\|up\] N number of lines
cursor \[first\|last\] | Move cursor to the \[first\|last\] line
select | Select line that cursor is currently on (i.e. add line that cursor is currently on to selected lines)
unselect | Unselect line that cursor is currently on
toggle-selection | Toggle selection of line that cursor is currently on
select-all | Select all lines
unselect-all | Unselect all currently selected lines
exec -- \<COMMAND\> | Execute shell command and block until command terminates
exec -- \<COMMAND\> & | Execute shell command as background process, i.e. don't block until command terminates
help-\[show\|hide\|toggle\] | \[Show\|Hide\|Toggle\] the help menu that shows all activated keybindings

The shell command `COMMAND` will be executed in a subshell that has the environment variable `LINES` set to all selected lines or, if none are selected, the line the cursor is currently on.
If multiple lines are selected, they will be separated by a newline in `LINES`.

### Formatting with Field Separators and Selections

`watchbind` supports some extra formatting features reminiscent of the Unix `cut` command:

- **Field Separators**:
Define a separator/delimiter to segment your command's output into distinct fields.
Each separator will be replaced with an [elastic tabstop](https://nick-gravgaard.com/elastic-tabstops/), resulting in a "table"-like structure, similar to the `cut -d \<SEPARATOR\> -t` command.

- **Field Selections**:
Choose only specific fields to display.
You can specify a comma-separated list of the indexes (starting at index 1) for individual fields (`X`), ranges (`X-Y`), or the capture of all fields from X onwards (`X-`).
For instance, the field selection `1,3-4,6-` will display the first, third and fourth fields, as well as all fields from the sixth onwards.

**Important**: The `LINES` passed to the `exec --` operations will remain unformatted, i.e. will not have the separators replaced with elastic tabstops and will not have non-selected fields ommitted.

### Styling

Foreground colors, background colors and boldness of the line the cursor is on, the header lines and all other lines can be customized.

To see all available fields you can customize, run `watchbind -h`.
The names of the customization fields from the command-line options (e.g. `--cursor-fg blue`) are the same in the TOML config file (e.g. `cursor-fg = "blue"`).

All supported `COLOR` values:
```
white
black
red
green
yellow
blue
magenta
cyan
gray
dark_gray
light_red
light_green
light_yellow
light_blue
light_magenta
light_cyan
```

## Tips

### Keybindings on selected lines that delete some of the input lines

I define "deleting input lines" as executing a keybinding that changes the length of the watched command's output.
In other words:
If, after executing a keybinding, the watched command generates an output longer or shorter than before the keybinding, then that keybinding deletes input lines.

Why is this definition important?
Because the selected lines are only stored as indices and, therefore, have no association to the actual lines displayed in watchbind.

Here's an example that demonstrates what problems this can cause:
You select five lines and then, through a keybinding, execute a command that deletes these five lines.
The next time your watched command is executed, it will output five lines less (that are displayed in watchbind), since the five lines have been deleted.
The problem is that the indices of the deleted lines will still be marked as selected.
Therefore, five different lines, at the same indices as the deleted five lines, will now be selected, which is probably unwanted.

To solve this problem, the following keybinding format is recommended for keybindings that transform the input:
```toml
[keybindings]
"KEY" = [ "exec -- DELETE-OP", "reload", "unselect-all" ]
```

First, the selected lines are deleted using the `DELETE-OP` (e.g. `echo $LINES | xargs rm`).
Then, we want to see the updated output of the watched command that doesn't contain the deleted lines anymore, so we `reload`.
Finally, we want to remove our the selection of the now removed lines, so we call `unselect-all`.

### Piping

If you want to use pipes in your watched command on the command-line, make sure to escape the pipe symbol like so:
```
watchbind ls \| grep "test"
```
or put quotes around the watched command
```
watchbind "ls | grep test"
```
Otherwise, the shell will think you want to pipe the output of `watchbind exec -- ls` to `grep test`.

### Subshell

The commands you bind to keys will be executed in a subshell using `sh -c`.

This means you can run a command like 
```
watchbind --bind "enter:notify-send \$LINES" ls
```
and the environment variable `$LINES` will contain the line the cursor is currently on.

But note that 
```
watchbind --bind "enter:notify-send $LINES" ls
```
will not work as expected, because `$LINES` will be replaced in the shell you are running the `watchbind` command from.
