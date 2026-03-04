# Fluent

This is a general purpose hotkey plugin for [interception-tools](https://gitlab.com/interception/linux/tools).
It allows to remap arbitrary hotkeys with a configurable set of modifiers.

>This is still work in progress! If you have feature requests or bugs, feel free to write an issue!

## Installation

### Arch

Currently, is is available for Arch in the [AUR](https://aur.archlinux.org/packages/interception-fluent).
You can install it from there.

### Other Distros

Install [interception-tools](https://gitlab.com/interception/linux/tools) according to their readme for your distro.
Then, install Rust, check out this repository, build it and copy the executable to a folder on your `$PATH`.

```sh
git clone https://github.com/Lixissimus/fluent.git
cd fluent
cargo build --release
cp target/release/fluent /usr/bin
```

Copy `fluent.yaml` from the directory structure of the `data/` dir in this repo to the corresponding location in your system.
This ensures that `udevmon` (this tool is part of `interception-tools`) starts the plugin for every connected keyboard.

## Config

### Hotkey Config

The hotkey config is read from `/etc/interception/fluent.json`.
Here is an example config for a start:

```json
{
    "modifiers": [
        "ctrl_left",
        "alt_left",
        "shift_left",
        "ctrl_right",
        "alt_right",
        "shift_right",
        "capslock"
    ],
    "mappings": [
        {
            "on": [
                "capslock",
                "j"
            ],
            "send": [
                "left"
            ]
        },
        {
            "on": [
                "alt_left",
                "c"
            ],
            "send": [
                "ctrl_left",
                "c"
            ]
        }
    ]
}
```

#### Modifiers

`modifiers` is an optional element.
If not specified, this defaults to `ctrl_left/right`, `alt_left/right`, and `shift_left/right`.
Modifiers are important, because a hotkey triggers as soon as the first non-modifier key is pressed.
Every key on the keyboard can be configured as a modifier.
Note: Currently, modifiers that are part of any configured hotkey are not forwarded to the system currently while a match is still possible.
E.g. in the above example config, `alt_left` will not be forwarded when pushed alone.
Only once a non-modifier that is not `c` is pressed, `alt_left` is sent to the system, because it is not handled by `fluent`.

#### Mappings

`mappings` is an array of hotkey objects, where each hotkey defines a trigger sequence in `on`.
The trigger sequence must consist of any number of modifiers and a single non-modifier key.
The `send` sequence will be sent when the full `on` sequence matches.

If you are unsure about the namings of the keys, check `src/keys.rs`.

### Testing

Before enabling the background service, it is recommended to test your config in a foreground process where you directly see errors.
Run the following as root.
Switch into a root shell using `sudo su`, because both `intercept` and `uinput` need to run as root.
Search for your keyboard by running `ls -l /dev/input/by-id`.
Also, add the `sleep 1` in the beginning of the command, otherwise it might already process the key-up event of your return key, which results in weird behavior.

```sh
sleep 1 && intercept -g /dev/input/by-id/YOUR_KEYBOARD-event-kbd | fluent | uinput -d /dev/input/by-id/usb-Razer_Razer_BlackWidow_Elite-event-kbd 
```

If you want to log what the plugin is doing, check the output of `ls -l /dev/input` _before_ running the command.
Then run the command and check the output again.
There should be one new `/dev/input/eventXX`.
This is the virtual device created by `uinput`.
You can now run `sudo evtest /dev/input/eventXX` and you will see all the events produced by your config.

## Running as Background Service

`udevmon` can be used to automatically start the plugin for all connected keyboards.
See the `interception-tools` documentation for more details.
A default configuration comes with the plugin and is installed in `/etc/interception/udevmon.d/fluent.yaml`.

Start the background service:

```sh
sudo systemctl start udevmon.service
```

Enable automatic start on system start:

```sh
sudo systemctl enable udevmon.service
```

### Keyboard Selection

By default, one instance of the plugin is started for each connected keyboard.
This is configured in `/etc/interception/udevmon.d/fluent.yaml`.
Check the documentation of the `interception-tools` for more details if you want to modify this file.

## TODO

- notify interception maintainer of new plugin
- allow to send multiple key combination
- allow to trigger commands
- vizualize state machine for documentation
- suppress all non-mapped keys (for vim mode)
- disable depending on active window title
- find better config format than json
- publish current mode on DBus to enable plugins e.g. for waybar to show current mode
