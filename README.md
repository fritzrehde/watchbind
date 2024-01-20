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
- [How it works](#how-it-works)
- [Configuration](#configuration)
  - [Keybindings](#keybindings)
  - [Styling](#styling)
  - [Formatting with Field Separators and Field Selections](#formatting-with-field-separators-and-field-selections)
  - [State Management](#state-management)
- [Tips](#tips)


## Features

- **Customizability**: All keybindings and styles (colors and boldness) are customizable.
- **Flexibility**: Settings can be configured through CLI arguments, a local TOML config file, a global TOML config file, or a combination of these.
- **Speed**: Written in asynchronous Rust with [Tokio](https://tokio.rs/).


## Installation

### From binaries

The [releases page](https://github.com/fritzrehde/watchbind/releases) contains pre-compiled binaries for Linux, macOS and Windows.

### From [crates.io](https://crates.io/crates/watchbind)

```sh
cargo install watchbind
```

### Distro Packages

[![Packaging status](https://repology.org/badge/vertical-allrepos/watchbind.svg)](https://repology.org/project/watchbind/versions)

#### Arch Linux

`watchbind` can be installed from the [extra repository](https://archlinux.org/packages/extra/x86_64/watchbind) using [pacman](https://wiki.archlinux.org/title/Pacman):

```sh
pacman -S watchbind
```

#### Alpine Linux

`watchbind` is available for [Alpine Edge](https://pkgs.alpinelinux.org/packages?name=watchbind&branch=edge). It can be installed via [apk](https://wiki.alpinelinux.org/wiki/Alpine_Package_Keeper) after enabling the [testing repository](https://wiki.alpinelinux.org/wiki/Repositories).

```sh
apk add watchbind
```


## How it works

Watchbind is a command-line tool that aims to help you build custom TUIs from static CLI commands very easily.
It works by specifying a "watched command" that outputs some lines to stdout that you want to observe.
Then, we make this static output **dynamic** by re-executing it at a specified watch rate, and we make the TUI **interactive** through custom keybindings that can operate on the individual output lines.


## Configuration

There are several ways to configure watchbind's settings:
1. **CLI arguments** (see `watchbind -h` for all available arguments).
2. A **local TOML config file**, specified with `watchbind --local-config-file <FILE>`, that applies settings only to this watchbind instance.
3. A **global TOML config file**, located either in the user-specified `WATCHBIND_CONFIG_DIR` environment variable or in the default config directory (see `watchbind --help` for the OS-specified default config directory), that applies settings to all watchbind instances.

All configuration ways can be used at the same time, and `watchbind` will determine which settings to use according to the following configuration hierarchy:
```
CLI arguments > local TOML config file > global TOML config file
```
where `a > b` means: If the config setting `X` is configured in both `a` and `b`, the value of `X` from `a` is used.


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

<!-- TODO: Make table of toml config reference name,example,description -->

All supported `OP` values:

Operation | Description
:-- | :--
`exit` | Quit watchbind.
`reload` | Reload the watched command manually, resets interval timer.
`cursor [down\|up] <N>` | Move cursor \[down\|up\] N number of lines.
`cursor [first\|last]` | Move cursor to the \[first\|last\] line.
`select` | Select line that cursor is currently on (i.e. add line that cursor is currently on to selected lines).
`unselect` | Unselect line that cursor is currently on.
`toggle-selection` | Toggle selection of line that cursor is currently on.
`select-all` | Select all lines.
`unselect-all` | Unselect all currently selected lines.
`exec -- <CMD>` | Execute `CMD` and block until termination.
`exec & -- <CMD>` | Execute `CMD` as background process, i.e. don't block until command terminates.
`exec tui -- <TUI-CMD>` | Execute a `TUI-CMD` that spawns a TUI (e.g. text editor). Watchbind's own TUI is replaced with `TUI-CMD`'s TUI until `TUI-CMD` terminates. Note that `TUI-CMD` must spawn a full-screen TUI that covers the entire terminal, otherwise undefined behaviour will ensue.
`set-env <ENV> -- <CMD>` | Blockingly execute `CMD`, and save its output to the environment variable `ENV`.
`unset-env <ENV> -- <CMD>` | Unsets environment variable `ENV`.
`help-[show\|hide\|toggle]` | \[Show\|Hide\|Toggle\] the help menu that shows all activated keybindings.

All `CMD` and `TUI-CMD` shell commands will be executed in a subshell (i.e. `sh -c "CMD"`) that has some environment variables set.
The environment variable `line` is set to the line the cursor is on.
The environment variable `lines` set to all selected lines, or if none are selected, the line the cursor is currently on.
All set environment variables `ENV` will be made available in all future spawned commands/processes, including the watched command, any executed subcommands, as well as commands executed in `set-env` operations.
If multiple lines are selected, they will be separated by newlines in `lines`.

### Styling

Foreground colors, background colors and boldness can be customized.
These styling options are available for:
- The line the cursor is currently on with `cursor-[fg|bg|boldness]`.
- The header lines with `header-[fg|bg|boldness]`.
- All other lines with `non-cursor-non-header-[fg|bg|boldness]`.
- The selection indicator with `selected-bg`.

The names of the customization fields from the command-line options (e.g. `--cursor-fg blue`) are the same in the TOML config file (e.g. `cursor-fg = "blue"`).

Furthermore, `watchbind` also supports styling according to ANSI codes in the input text.

All supported `COLOR` values:
```sh
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
reset        # Reset the fg and bg
unspecified  # Don't applying any styling => use style from ANSI input text
```

All supported `BOLDNESS` values:
```sh
bold         # Make everything bold
non-bold     # Make sure nothing is bold (i.e. remove any bold styling from input ANSI)
unspecified  # Don't applying any styling => use style from ANSI input text
```

### Formatting with Field Separators and Field Selections

`watchbind` supports some extra formatting features reminiscent of the Unix `cut` command:

- **Field Separators**:
Define a separator/delimiter to segment your command's output into distinct fields.
Each separator will be replaced with an [elastic tabstop](https://nick-gravgaard.com/elastic-tabstops/), resulting in a "table"-like structure, similar to the `cut -d <SEPARATOR> -t` command.

- **Field Selections**:
Choose only specific fields to display.
You can specify a comma-separated list of the indexes (starting at index 1) for individual fields (`X`), ranges (`X-Y`), or the capture of all fields from X onwards (`X-`).
For instance, the field selection `1,3-4,6-` will display the first, third and fourth fields, as well as all fields from the sixth onwards.

**Important**: The `lines` passed to the `exec --` operations will remain unformatted, i.e. will not have the separators replaced with elastic tabstops and will not have non-selected fields ommitted.

### State management

The `set-env` and `unset-env` operations allow you to manage state through environment variables.
Additionally, you can use the `initial-env` option to specify a list of `set-env` commands that will be executed **before** the first execution of the watched command.
This powerful combination allows you to set some initial state with `initial-env`, reference that state directly in the watched command, and update the state with keybindings at runtime with `set-env`.


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

First, the selected lines are deleted using the `DELETE-OP` (e.g. `echo $lines | xargs rm`).
Then, we want to see the updated output of the watched command that doesn't contain the deleted lines anymore, so we `reload`.
Finally, we want to remove our the selection of the now removed lines, so we call `unselect-all`.

### Piping

If you want to use pipes in your watched command on the command-line, make sure to escape the pipe symbol like so:
```sh
watchbind ls \| grep "test"
```
or put quotes around the watched command
```sh
watchbind "ls | grep test"
```
Otherwise, the shell will think you want to pipe the output of `watchbind exec -- ls` to `grep test`.

### Subshell

The commands you bind to keys will be executed in a subshell using `sh -c`.

This means you can run a command like 
```sh
watchbind --bind "enter:notify-send \$lines" ls
```
and the environment variable `$lines` will contain the line the cursor is currently on.

But note that 
```sh
watchbind --bind "enter:notify-send $lines" ls
```
will not work as expected, because `$lines` will be replaced in the shell you are running the `watchbind` command from.
