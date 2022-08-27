This project is still a work in progress.

# Watchbind

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
