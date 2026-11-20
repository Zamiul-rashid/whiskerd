#!/bin/sh
# Apply the udev rule to receivers that are already plugged in, and start whiskerd at login.
udevadm control --reload-rules >/dev/null 2>&1 || true
udevadm trigger --subsystem-match=hidraw --action=change >/dev/null 2>&1 || true
systemctl --global enable whiskerd.service >/dev/null 2>&1 || true
cat <<'MSG'
whiskerd is installed and starts automatically at your next login.
Start it now:           systemctl --user daemon-reload && systemctl --user restart whiskerd
GNOME top bar:          log out and back in, then: gnome-extensions enable whiskerd@zamiul-rashid
Other bars (Waybar...): read $XDG_RUNTIME_DIR/whiskerd.json, or run: whiskerd --output -
MSG
