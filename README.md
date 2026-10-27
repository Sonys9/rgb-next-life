# rgb-next-life

> Bring a next life to your cheap analog rgb!

**rgb-next-life** is an optimized, fast, asynchronous, bare-metal controller for you analog rgb! Using this firmware, you can control your RGB easier than ever before.

## Why?
- **Pure Asynchronous Rust**: Powered by the Embassy executor for fastest usage.
- **Small amounts of allocations**: Forget about random crashes.
- **Triad Control**: You can control it by using the screen, by web panel or by manual editing the color by sending UDP packets (requires Wi-Fi).

## FAQ

#### What if my PC will stop sending UDP packets?
The controller will automaticly switch to selected default color mode after 60 seconds of inactivity.

#### What if I don't have a display?
The controller automatically hosts an access point. You can connect to it, provide your Wi-Fi credentials, and then just head over to rgbnextlife.local.

#### What if the board fails to boot?
The controller probably hit a critical hardware error or read corrupted flash payload data and caught itself in a crash loop. Create an issue on GitHub with your serial logs attached, then clear the memory blocks entirely by executing:
```bash
esptool --chip esp32 --port PORT_HERE (/dev/tty\*\*\* or COM\*\*) erase_flash
```

# TODO:
- Storage:
    - [UNDONE] Custom simple FS
- Network:
    - [DONE] AP Creation
    - [DONE] Web Server
    - [UNDONE] Wi-Fi connection
    - [UNDONE] Listening for UDP packets and parsing 3 incoming bytes (RGB)
    - [UNDONE] Web panel (with stuff that screen has)
- Screen:
    - [UNDONE] Connect, creating easy way to include other screen drivers (traits and etc), different screen sizes
    - [UNDONE] Status, current color
    - [UNDONE] The pet!
    - [UNDONE] Autonomic colors choose menu
- RGB:
    - [UNDONE] Connect and try basic colors
    - [UNDONE] Connect it with the screen and network!