# Fluent

execute as root:
```
sleep 1 && intercept -g /dev/input/by-id/usb-Razer_Razer_BlackWidow_Elite-event-kbd | ./target/debug/fluent | uinput -d /dev/input/by-id/usb-Razer_Razer_BlackWidow_Elite-event-kbd 
```