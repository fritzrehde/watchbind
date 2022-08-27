This project is still a work in progress.

# Watchbind

## Shell

The command you pass to `watchbind` will be executed in a shell using `sh -c`.
This means you can run a command like 
``watchbind --bind "enter:notify-send \$LINE" ls``
and it will replace `$LINE` with the selected line.
But note that 
``watchbind --bind "enter:notify-send $LINE" ls``
will not work as expected, as `$LINE` would be replaced in the shell you are running the `watchbind` command from instead of within `watchbind` itself.
