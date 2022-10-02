This project is still a work in progress.

# Watchbind

<!-- ![watchbind demo](https://raw.githubusercontent.com/fritzrehde/i/master/watchbind/demo.gif) -->
![demo](https://raw.githubusercontent.com/fritzrehde/i/master/watchbind/screenshot-light.png#gh-light-mode-only) ![demo](https://raw.githubusercontent.com/fritzrehde/i/master/watchbind/screenshot-dark.png#gh-dark-mode-only)

## Piping

If you want to use pipes in your command, make sure to escape the pipe symbol like so:
```
watchbind ls \| grep "test"
```
or put quotes around the command
```
watchbind "ls | grep test"
```

## Shell

The commands you bind to keys will be executed in a subshell using `sh -c`.

This means you can run a command like 
```
watchbind --bind "enter:notify-send \$LINE" ls
```
and it will replace `$LINE` with the selected line in the subshell.

But note that 
```
watchbind --bind "enter:notify-send $LINE" ls
```
will not work as expected, because `$LINE` will be replaced in the shell you are running the `watchbind` command from.
