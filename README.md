# elgato-autolight

Automatically toggle Elgato lights when your Mac camera activates. Monitors macOS camera events and shells out to [elgato-light](https://github.com/wassimk/elgato-light) to control your lights.

### How It Works

The tool watches the macOS system log for UVC camera power events. When the camera turns on (e.g., joining a video call), it runs `elgato-light on`. When the camera turns off, it runs `elgato-light off`.

### Install

```shell
brew install wassimk/tap/elgato-autolight
```

This will also install `elgato-light` as a dependency.

After installing, set it up as a background service:

```shell
elgato-autolight install
```

That's it. The light will now turn on and off automatically with your camera.

### Usage

```
$ elgato-autolight --help

Automatically toggle Elgato lights when your Mac camera activates

Usage: elgato-autolight <COMMAND>

Commands:
  start      Run the camera monitor in the foreground
  install    Install the LaunchAgent for automatic startup
  uninstall  Uninstall the LaunchAgent
  status     Show running state, config, and log paths

Options:
  -h, --help     Print help
  -V, --version  Print version
```

Run the monitor in the foreground for testing:

```shell
elgato-autolight start
elgato-autolight start --verbose
```

Install as a LaunchAgent that starts automatically on login:

```shell
elgato-autolight install
elgato-autolight install --force   # overwrite existing
```

Remove the LaunchAgent:

```shell
elgato-autolight uninstall
```

Check the current state:

```shell
elgato-autolight status
```

### Configuration

Create `~/.config/elgato-autolight/config.toml` to override defaults:

```toml
brightness = 10          # 0-100, default 10
temperature = 5000       # 2900-7000K, default 5000
# light = "Key Light"    # --light flag passed to elgato-light
# ip_address = "1.2.3.4" # --ip-address flag passed to elgato-light
```

If the file is missing, defaults are used. No config file is created automatically.

### Logs

When running as a LaunchAgent, logs are written to:

- `~/Library/Logs/elgato-autolight/stdout.log`
- `~/Library/Logs/elgato-autolight/stderr.log`

### Troubleshooting

Verify the service is running:

```shell
elgato-autolight status
```

Test the monitor interactively with verbose output:

```shell
elgato-autolight start --verbose
```

Then open Photo Booth or FaceTime to trigger the camera.

If `elgato-light` is not found, install it:

```shell
brew install wassimk/tap/elgato-light
```
