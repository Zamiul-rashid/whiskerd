#!/usr/bin/env bash
# Build and install whiskerd for the current user from a git checkout.
#   ./install.sh              install (asks for sudo once, for the udev rule)
#   ./install.sh --uninstall  remove everything this script installed
set -euo pipefail
cd "$(dirname "$0")"

UUID="whiskerd@zamiul-rashid"
BIN="$HOME/.local/bin/whiskerd"
UNIT="$HOME/.config/systemd/user/whiskerd.service"
EXT="$HOME/.local/share/gnome-shell/extensions/$UUID"
RULE="/etc/udev/rules.d/70-whiskerd.rules"

if [[ "${1:-}" == "--uninstall" ]]; then
    systemctl --user disable --now whiskerd.service 2>/dev/null || true
    gnome-extensions disable "$UUID" 2>/dev/null || true
    rm -f "$BIN" "$UNIT"
    rm -rf "$EXT"
    systemctl --user daemon-reload
    if [[ -e "$RULE" ]]; then
        sudo rm -f "$RULE"
        sudo udevadm control --reload-rules
    fi
    echo "whiskerd removed."
    exit 0
fi

if ! command -v cargo >/dev/null; then
    echo "whiskerd needs Rust to build: https://rustup.rs (or use the .deb/.rpm/Arch packages)" >&2
    exit 1
fi

cargo build --release --locked
install -Dm755 target/release/whiskerd "$BIN"
mkdir -p "$(dirname "$UNIT")"
sed "s|/usr/bin/whiskerd|$BIN|" packaging/whiskerd.service > "$UNIT"

echo "Installing the udev rule (needs sudo once) so whiskerd can read the receivers without root:"
"$BIN" --udev-rules | sudo install -Dm644 /dev/stdin "$RULE"
sudo udevadm control --reload-rules
sudo udevadm trigger --subsystem-match=hidraw --action=change

mkdir -p "$EXT"
cp gnome-extension/* "$EXT/"

systemctl --user daemon-reload
systemctl --user enable whiskerd.service
systemctl --user restart whiskerd.service

echo
echo "whiskerd is running. Battery file: \${XDG_RUNTIME_DIR}/whiskerd.json"
if command -v gnome-extensions >/dev/null; then
    if gnome-extensions enable "$UUID" 2>/dev/null; then
        echo "GNOME extension enabled."
    else
        echo "GNOME: log out and back in once, then run: gnome-extensions enable $UUID"
    fi
fi
