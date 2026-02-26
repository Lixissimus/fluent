# Fluent

execute as root:
```
sleep 1 && intercept -g /dev/input/by-id/usb-Razer_Razer_BlackWidow_Elite-event-kbd | ./target/debug/fluent | uinput -d /dev/input/by-id/usb-Razer_Razer_BlackWidow_Elite-event-kbd 
```

## TODO

- add delay when sending key events and timestamp and EV_SYN: https://gitlab.com/interception/linux/tools#correct-synthesization-of-event-sequences
- create arch package
- enable logging
- allow to not only send keys but also trigger commands
- currently interferes with ctrl + mouse wheel for zoom :(
- vizualize state machine for documentation
