# Pomo

Simple command line pomodoro timer.


## Usage

```
🍅 Simple CLI pomodoro timer written in rust.

Usage: pomo <COMMAND>

Commands:
  start    Start a new pomodoro
  status   Prints the current pomo
  watch    Watch current pomo and print current state every second
  stop     Stops the pomo.
  pause    Pauses the pomo, can be resumed with 'unpause'
  unpause  Unpauses the pomo
  info     Print list of current pomos
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

Run `pomo help <COMMAND>` to get help for one of the commands.

### `pomo start`

The subcommand `start` accepts a pomo definition: a string with a default value of `4p45b10`.
The pomo definition has the format `[<repetitions>][p<work-duration>][b<pause-duration>]`.
Each of the three segments is optional.

#### `pomo start --until`

This flag allows the user to set an end time. The work duration and number of repetitions will get changed so that 
the pomodoro timer finishes exactly at the specified time. The work duration is modified as little as possible from the manually set value, the manually set repetitions are ignored.

**Example**
`pomo start 4p30b5 --until 16:00`


## Tmux integration

`pomo` works great with tmux. I use it in the status bar to always show me the currently active pomodoro. Use the following line of tmux config to 
display the current state:

```tmux.config
set -g status-right ' #(pomo status) '
```

Obviously you can use the `#(pomo status)` command wherever you want.


## OBS integration

If you want to display the current pomodoro in [OBS](https://obsproject.com/), then run the command `pomo watch [path/to/pomodoro.txt]` and keep it running.
`pomo` will update the pomodoro text file every second. Now configure a text source in OBS to read from a file to show it on the screen.

## Installation

Clone this repo and install with `cargo install --path .` or use the following cargo command:

```sh
cargo install --git https://github.com/Tiim/pomo.git
```
