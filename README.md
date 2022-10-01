This project is still a work in progress.

# Watchbind

![watchbind demo](https://raw.githubusercontent.com/fritzrehde/i/master/watchbind/demo.gif)

https://user-images.githubusercontent.com/80471265/193427365-43a3bdfd-d4ec-4d10-8d16-a75480ae361b.mp4

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
