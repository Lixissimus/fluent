# Fluent

execute as root for debugging:

```
sleep 1 && intercept -g /dev/input/by-id/usb-Razer_Razer_BlackWidow_Elite-event-kbd | ./target/debug/fluent | uinput -d /dev/input/by-id/usb-Razer_Razer_BlackWidow_Elite-event-kbd 
```

## TODO

- create arch package
- allow to not only send keys but also trigger commands
- currently interferes with ctrl + mouse wheel for zoom :(
- vizualize state machine for documentation
