# Demo

The README GIF is rendered with [VHS](https://github.com/charmbracelet/vhs) from `elgato-autolight.tape`.

## Requirements

- `vhs`
- `elgato-autolight` and `elgato-light` on your `PATH`
- `python3`
- JetBrainsMono Nerd Font

## Rendering

From the repo root:

```shell
vhs demo/elgato-autolight.tape
```

This overwrites `demo/elgato-autolight.gif`.

## How it works

No camera or real light is needed, and the real launchd is never touched.

- The tape puts `demo/bin` first on `PATH`, so the CLI runs the fake `log` and `launchctl` in it.
- `bin/launchctl` handles `bootstrap` by running `elgato-autolight start` in the background with output going to the usual log files, and answers `list` and `bootout` for that process.
- `bin/log` stands in for macOS `log stream`. It plays a camera on event after `DEMO_CAMERA_ON_AFTER` seconds (15 in the tape, so it lands while the log is followed on screen) and an off event 5 seconds later.
- `fake_light.py` serves the Elgato `/elgato/lights` API on `127.0.0.1:19123`, writes itself to the `elgato-light` discovery cache, and prints a dimmed line each time its state changes.
- The tape points `HOME` at a temp directory first, so the config, LaunchAgent plist, logs, and cache all live there. A hidden shell function rewrites that temp path to `~` in the CLI output.
