# Watchbind

*Turn the output of any command into a powerful TUI with custom keybindings.*

![screenshot](https://raw.githubusercontent.com/fritzrehde/i/master/watchbind/screenshot-light.png#gh-light-mode-only)
![screenshot](https://raw.githubusercontent.com/fritzrehde/i/master/watchbind/screenshot-dark.png#gh-dark-mode-only)

## Table of Contents

- [Features](#features)
- [Installation](#installation)
- [Customizations](#customizations)
- [Tips](#tips)

## Features

- **Customizable**: all keybindings and styles (colors and boldness) are customizable
- **Flexible**: specify settings using cli options, a toml config file or both
- **Speed**: written completely in rust with speed in mind

## Installation

### Build from source

```shell
git clone https://github.com/fritzrehde/watchbind.git
cd watchbind
cargo build
```

You can also run `cargo run -- -c examples/test-config.toml` to see a simple demo of `watchbind`.

### From [crates.io](https://crates.io/crates/watchbind)

```shell
cargo install watchbind
```

### AUR

To be added.

## Customizations

There are several ways to customize the settings:
1. A toml config file, specified with `watchbind --config-file <FILE>`, overrides all default settings (Example: [test-config.toml](examples/test-config.toml)).
2. The command-line options override all other settings (i.e. all toml and default settings).

All ways of configuring `watchbind` (toml and cli options) can be used at the same time, and `watchbind` will automatically figure out which settings to use according to the above hierarchy.

Personally, I recommend using the cli options for small one liners and a toml config file for more complex scripts.

### Keybindings

#### Via command-line arguments

On the command line, you can specify keybindings with the option `--bind "KEY:OPS[,KEY:OPS]*"`, where `OPS` is a list of operations `OP` that are bound to `KEY`.
One `KEY` can be bound to multiple operations, therefore, the syntax for each list of operations (`OPS`) is `OP[+OP]*`.
The operations are seperated by `+` and executed in succession (one after the other).

**TLDR**: operations are seperated by `+`, keybindings are seperated by `,`

#### Via toml config file

In a toml config file, specify keybindings like so:
```toml
[keybindings]
"KEY" = [ "OP" ]
"KEY" = [ "OP", "OP", "OP" ]
"KEY" = [ 
  "OP",
  "OP"
]
```

This syntax differs from the command-line syntax because using the toml array feature is more expressive and more native to the toml file format.
Furthermore, this allows you to use the `+` character in your commands.
It also doesn't require escaping shell specific characters like `$` in  (read more [in this section](#subshell)).

You can find some keybinding examples in [`test-config.toml`](examples/test-config.toml).

<details>
<summary>All supported KEY values</summary>
Format: `MODIFIER+CODE` or `CODE`

MODIFIER:
```
alt
ctrl
```

CODE:
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
[any single character]
```
</details>

<details>
<summary>All supported OP values</summary>

Operation | Action
:-- | :--
exit | Quit watchbind
reload | Reload the input command manually, resets interval timer
down | Go down one line (i.e. move cursor to the next line)
down \<STEPS\> | Go down STEPS number of lines
up | Go up one line (i.e. move cursor to the previous line)
up \<STEPS\> | Go up STEPS number of lines
first | Go to the first line
last | Go to the last line
select | Select line that cursor is currenly on (i.e. add line that cursor is currently on to selected lines)
unselect | Unselect line that cursor is currently on
select-toggle | Toggle selection of line that cursor is currently on
select-all | Select all lines
unselect-all | Unselect all currently selected lines
COMMAND | Execute shell command and block until command terminates
COMMAND & | Execute shell command as background process, i.e. don't block until command terminates

COMMAND will be executed in a subshell that has the environment variable `LINES` set to all selected lines or, if none are selected, the line the cursor is currently on.
If multiple lines are selected, they will be seperated by a newline in `LINES`.
</details>

### Style

Foreground colors, background colors and boldness of the line the cursor is on and all other lines can be customized.

To see all available fields you can customize, run `watchbind -h`.
The names of the customization fields from the command-line options (e.g. `--fg+ blue`) are the same in the toml config file (e.g. `"fg+" = "blue"`).

**Note**: Since a field name like `fg+` contains the toml special character `+`, the field name has to be put in quotations marks like `"fg+"`.

<details>
<summary>All supported COLOR values</summary>

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
</details>

## Tips

### Keybindings on selected lines that delete some of the input lines

I define "deleting input lines" as executing a keybinding that changes the length of the input command's output.
In other words:
If, after executing a keybinding, the input command generates an output longer or shorter than before the keybinding, then that keybinding deletes input lines.

Why is this definition important?
Because the selected lines are only stored as indices and, therefore, have no association to the actual lines displayed in watchbind.

Here's an example that demonstrates what problems this can cause:
You select five lines and then, through a keybinding, execute a command that deletes these five lines.
The next time your input command is called, it will output five lines less (that are displayed in watchbind), since the five lines have been deleted.
The problem is that the indices of the deleted lines will still be marked as selected.
Therefore, five different lines, at the same indices as the deleted five lines, will now be selected, which is probably unwanted.

To solve this problem, the following keybinding format is recommended for keybindings that transform the input:
```toml
[keybindings]
"KEY" = [ "DELETE-OP", "reload", "unselect-all" ]
```

First, the selected lines are deleted using the `DELETE-OP` (e.g. `echo $LINES | xargs rm`).
Then, we want to see the updated output of the input command that doesn't contain the deleted lines anymore, so we `reload`.
Finally, we want to remove our the selection of the now removed lines, so we call `unselect-all`.

### Piping

If you want to use pipes in your command on the command line, make sure to escape the pipe symbol like so:
```
watchbind ls \| grep "test"
```
or put quotes around the command
```
watchbind "ls | grep test"
```
Otherwise, the shell will think you want to pipe the output of `watchbind ls` to `grep test`.

### Subshell

The commands you bind to keys will be executed in a subshell using `sh -c`.

This means you can run a command like 
```
watchbind --bind "enter:notify-send \$LINES" ls
```
and the environment variable `$LINE` will contain the line the cursor is currently on.

But note that 
```
watchbind --bind "enter:notify-send $LINES" ls
```
will not work as expected, because `$LINES` will be replaced in the shell you are running the `watchbind` command from.
