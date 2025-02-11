swaywsr - sway workspace renamer
======

`swaywsr` is a small program that uses [Sways's](https://swaywm.org/) [IPC Interface](https://github.com/swaywm/sway/blob/master/sway/sway-ipc.7.scd)
to change the name of a workspace based on its contents.

It is a port from [Daniel Berg's (roosta)](https://github.com/roosta) [i3wsr](https://github.com/roosta/i3wsr) which I also contributed to. Most of the code is the same.

## Details

The chosen name for a workspace is a composite of the `app_id` (wayland native) or `WM_CLASS` X11 window
property for each window in a workspace. In action it would look something like this:

![](https://raw.githubusercontent.com/pedroscaff/swaywsr/master/assets/preview.png)

## Installation
Build a release binary,

```sh
cargo build --release
```

## Usage
Just launch the program and it'll listen for events if you are running sway.
Another option is to put something like this in your sway config

```
exec_always pgrep -x swaywsr > /dev/null && pkill -x swaywsr; $PATH_TO_RELEASE_BINARY
```

### Options


You must provide a config file that is passed using the `--config path_to_file.toml` option. The `toml` file has four fields:
- `icons` to assign icons to classes
- `aliases` to assign alternative names to be displayed 
- `general` to assign the separator, the default icon and the protected workspace ending char.
- `options` to assign additional flags available in the cli interface. Replace hyphens from cli with underscores, e.g. `--no-names` would be `no-names` in the config file.

You can configure icons for the respective classes, a very basic preset for font-awesome is configured, to enable it use the option `--icons awesome` (requires font-awesome to be installed).

If you have icons and don't want the names to be displayed, you can use the `--no-names` flag.

Workspace name can be protected from dynamic renaming using a trailing character. It defaults to '.', but can be overwritten using the option `ignore-char`.

Example config can be found in `assets/example_config.toml`

```toml
[icons]
# font awesome
TelegramDesktop = ""
Firefox = ""
Alacritty = ""
Thunderbird = ""
# smile emoji
MyNiceProgram = "😛"

[aliases]
TelegramDesktop = "Telegram"
"Org.gnome.Nautilus" = "Nautilus"

[general]
seperator = ""
ignore-char = "#"

[options]
no-names = true
remove-duplicates = true
```

For an overview of available options

```shell
$ swaywsr -h
swaywsr - sway workspace renamer 1.1.0
Pedro Scaff <pedro@scaff.me>

USAGE:
    swaywsr [FLAGS] [OPTIONS]

FLAGS:
    -h, --help                 Prints help information
    -n, --no-names             Set to no to display only icons (if available)
    -r, --remove-duplicates    Remove duplicate entries in workspace
    -V, --version              Prints version information

OPTIONS:
    -c, --config <config>    Path to toml config file
    -i, --icons <icons>      Sets icons to be used [possible values: awesome]
```

## Configuration

This program depends on numbered workspaces, since we're constantly changing the
workspace name. So your sway configuration need to reflect this:

```
bindsym $mod+1 workspace number 1
```

If you don't necessarily bind your workspaces to only numbers, or
you want to keep a part of the name constant you can do like this:

```
bindsym $mod+q workspace number 1:[Q]
```

This way the workspace would look something like this when it gets changed:

```
1:[Q] Emacs|Firefox
```
You can take this a bit further by using a bar that trims the workspace number and be left with only
```
[Q] Emacs|Firefox
```

## Contributors
* [Pedro Scaff (pedroscaff)](https://github.com/pedroscaff)
* [Raphaël Gallais-Pou (GallaisPoutine)](https://github.com/GallaisPoutine)

## Attribution
Thanks [Daniel Berg (roosta)](https://github.com/roosta) for the original [i3wsr](https://github.com/roosta/i3wsr) implementation. This program would not be possible without
[swayipc-rs](https://github.com/JayceFayne/swayipc-rs),
a rust library for controlling sway-wm through its IPC interface.
