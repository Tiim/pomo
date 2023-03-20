# Pomo

Simple command line pomodoro timer.


## Tmux integration

`pomo` works great with tmux. I use it in the status bar to always show me the currently active pomodoro. Use the following line of tmux config to 
display the current state:

```tmux.config
set -g status-right ' #(pomo status) '
```

Obviously you can use teh `#(pomo status)` command wherever you want.


## OBS integration

If you want to display the current pomodoro in [OBS](https://obsproject.com/), then run the command `pomo watch [path/to/pomodoro.txt]` and keep it running.
`pomo` will update the pomodoro text file every second. Now configure a text source in OBS to read from a file to show it on the screen.
