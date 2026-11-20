# whiskerd

Wireless mouse battery levels in your Linux status bar, without vendor software.

**Website:** https://zamiul-rashid.github.io/whiskerd/ · **Does my mouse work?** [Compatible mice](docs/compatible-mice.md)

Many 2.4 GHz mice never report their battery to Linux. whiskerd is a tiny daemon that talks to
their USB dongles directly, read-only. It publishes the levels as a small JSON file, and a GNOME
Shell extension shows them in the top bar.

- **About 2 MB of RAM and no CPU while idle.** It is one Rust process with a single `poll()` loop:
  no threads, no async runtime, and `libc` is its only dependency.
- **No root at runtime.** A udev rule gives the logged-in user access to the supported dongles only.
- **Hotplug-aware.** Plug a dongle in and its mouse shows up immediately.
- **Read-only.** whiskerd never changes any mouse setting.

## Supported devices

| Receiver (USB ID) | Mice | Status |
|---|---|---|
| `248a:5b2f` "SSCYPL Wireless-Receiver" | Ajazz AJ179P, plus other Compx-based mice (Delux M800 Pro, Genesis Zircon 500, Inphic, ...) | verified |
| `248a:5b2e`, `331a:5024` (wired) | the same family over USB cable | community-reported |
| `3434:d031` Keychron Link (Type-C) | Keychron M6, M3, M1, M2, M7 ... | verified |
| `3434:d030` Keychron Link (Type-A) | as above | community-reported |
| anything in `/sys/class/power_supply` with scope `Device` | Logitech (hid-logitech-hidpp) and other mice the kernel already reads | via kernel |

**verified** means it has been tested on real hardware by this project. **community-reported** means
the protocol is documented by other projects for that ID. Run `whiskerd --list` for the current table
and `whiskerd --probe` to see what whiskerd finds on your machine.

The per-mouse list, with how sure we are about each one, is in
[docs/compatible-mice.md](docs/compatible-mice.md) and on the [website](https://zamiul-rashid.github.io/whiskerd/#mice).

whiskerd runs on Linux only for now.

## Install

### Ubuntu / Debian

```sh
curl -fsSL https://zamiul-rashid.github.io/whiskerd/whiskerd.asc | sudo gpg --dearmor -o /usr/share/keyrings/whiskerd.gpg
echo "deb [signed-by=/usr/share/keyrings/whiskerd.gpg] https://zamiul-rashid.github.io/whiskerd/apt stable main" | sudo tee /etc/apt/sources.list.d/whiskerd.list
sudo apt update && sudo apt install whiskerd
```

### Fedora

```sh
sudo curl -fsSL -o /etc/yum.repos.d/whiskerd.repo https://zamiul-rashid.github.io/whiskerd/rpm/whiskerd.repo
sudo dnf install whiskerd
```

### Arch Linux

Download `whiskerd-*.pkg.tar.zst` from the [releases](https://github.com/Zamiul-rashid/whiskerd/releases)
and run `sudo pacman -U whiskerd-*.pkg.tar.zst`, or build it from
[`packaging/arch/PKGBUILD`](packaging/arch/PKGBUILD) with `makepkg -si`.

### From source (any distro)

```sh
git clone https://github.com/Zamiul-rashid/whiskerd && cd whiskerd
./install.sh            # needs Rust (https://rustup.rs); asks for sudo once for the udev rule
./install.sh --uninstall
```

### After installing

1. Start the daemon: `systemctl --user restart whiskerd`. It also starts at every login.
2. GNOME: log out and back in once, then run `gnome-extensions enable whiskerd@zamiul-rashid`.

## How it works

| Family | How the battery is read |
|---|---|
| Compx (`248a:5b2f` ...) | Feature report `0x0C`, command `0x20` (status), sent every 5 minutes, or every 30 seconds while the mouse sleeps. A sleeping mouse is not woken: only the receiver answers. |
| Keychron Link (`3434:d031` ...) | The receiver does **not** answer battery requests. It pushes `54 e2 01 01 00 <pct> 02 02` whenever the mouse connects or wakes, and whiskerd listens for it. Right after login, the Keychron value appears the first time you use the mouse. |
| Kernel | Reads `/sys/class/power_supply/*/capacity`, updated on kernel change events. |

The output file is `$XDG_RUNTIME_DIR/whiskerd.json` (in RAM):

```json
{"devices":[{"id":"248a:5b2f@3-5.1","name":"Ajazz AJ179P","kind":"mouse","battery":74,"charging":false,"connected":true,"updated":1795190426}]}
```

`connected: false` means the mouse is asleep or out of range; `battery` then holds the last known
value. Bars other than GNOME can read this file, or run `whiskerd --output -` to get one JSON line
on stdout for every change (Waybar `custom` modules, polybar, eww ...).

## Adding a mouse

- **The receiver speaks a protocol whiskerd already knows:** add one line to `RECEIVERS` in
  [`src/devices.rs`](src/devices.rs). The udev rule is generated from that table.
- **Either way:** add the mouse to [`docs/compatible-mice.md`](docs/compatible-mice.md) and
  [`website/data/mice.toml`](website/data/mice.toml) with its evidence level.
- **A new protocol:** add a parser in `src/drivers/` with tests built from real captured bytes, then
  wire it up in `src/main.rs`.

Please open an issue with the output of `lsusb` and `whiskerd --probe` either way. The design notes,
including how these protocols were reverse-engineered, are in
[`docs/superpowers/specs/`](docs/superpowers/specs/).

## Website

The site lives in [`website/`](website/) (a [Zola](https://www.getzola.org/) project). Preview it
with `cd website && zola serve`. Every push to `main` that changes `website/` publishes it to GitHub
Pages ([`.github/workflows/pages.yml`](.github/workflows/pages.yml)), next to the apt and dnf
repositories.

## License

MIT, except the files credited below.

## Credits

- Mouse icon (`gnome-extension/mouse.png`, `website/static/img/mouse-*.png`, `website/static/favicon.png`):
  [Flaticon icon 1682033](https://www.flaticon.com/free-icon/mouse_1682033), used under the
  Flaticon free license with attribution.
- Fonts on the website: [Mochiy Pop One](https://fonts.google.com/specimen/Mochiy+Pop+One) and
  [Zen Maru Gothic](https://fonts.google.com/specimen/Zen+Maru+Gothic), SIL Open Font License 1.1.
