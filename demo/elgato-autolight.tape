Output "demo/elgato-autolight.gif"

Require elgato-autolight
Require elgato-light
Require python3

Set Shell "bash"
Set FontFamily "JetBrainsMono Nerd Font"
Set FontSize 16
Set Width 900
Set Height 550
Set Padding 20
Set Theme "Catppuccin Mocha"
Set TypingSpeed 50ms

# Sandbox HOME for the config, LaunchAgent, logs, and the fake light's cache.
# demo/bin goes first on PATH so `log` and `launchctl` are the fakes.
Hide
Type "export DEMO_HOME=$(mktemp -d) && mkdir -p $DEMO_HOME/Library/Caches/elgato-light $DEMO_HOME/Library/LaunchAgents $DEMO_HOME/.config/elgato-autolight"
Enter
Type "printf 'brightness = 30\ntemperature = 4000\n' > $DEMO_HOME/.config/elgato-autolight/config.toml"
Enter
Type "python3 demo/fake_light.py $DEMO_HOME/Library/Caches/elgato-light/target & sleep 1"
Enter
# The camera turns on once the log is being followed on screen
Type "export HOME=$DEMO_HOME PATH=$PWD/demo/bin:$PATH DEMO_CAMERA_ON_AFTER=15 PS1='$ '"
Enter
# Show paths under HOME as ~ instead of the temp directory
Type `elgato-autolight() { command elgato-autolight "$@" | sed "s|$HOME|~|g"; }; clear`
Enter
Sleep 500ms
Show

Type "elgato-autolight install"
Sleep 500ms
Enter
Wait+Screen /installed and loaded/
Sleep 2s

Type "elgato-autolight status"
Sleep 500ms
Enter
Wait+Screen /Logs:/
Sleep 4s

Ctrl+L
Type "tail -f ~/Library/Logs/elgato-autolight/stderr.log"
Sleep 500ms
Enter
Wait+Screen@30s /Camera ON/
Wait+Screen /on, 30%, 4000K/
Wait+Screen@15s /Camera OFF/
Wait+Screen /7C4F: off/
Sleep 3s

Hide
Ctrl+C
Type "launchctl bootout x; kill %1; rm -rf $DEMO_HOME"
Enter
