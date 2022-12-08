# Watchbind

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

You can find some keybinding examples in [`test-config.toml`](examples/test-config.toml).

<details>
<summary>All supported KEY values</summary>

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
unselect | Unselect the currently selected line
next | Select the next line
prev | Select the previous line
first | Select the first line
last | Select the last line
COMMAND | Execute shell command and block until command terminates
COMMAND & | Execute shell command as background process, i.e. don't block until command terminates

COMMAND will be executed in a subshell that has the environment variable `LINE` set to the currently selected line.
</details>

### Style

Foreground colors, background colors and boldness of the selected line and all unselected lines can be customized.

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
watchbind --bind "enter:notify-send \$LINE" ls
```
and the environment variable `$LINE` will contain the selected line.

But note that 
```
watchbind --bind "enter:notify-send $LINE" ls
```
will not work as expected, because `$LINE` will be replaced in the shell you are running the `watchbind` command from.
