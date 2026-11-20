# whiskerd: mouse battery in the GNOME top bar (design)

Date: 2026-11-20 · Status: draft for review

## Goal

Show the battery level of the user's wireless mice in the GNOME top bar on Linux, without vendor
software, while using almost no RAM or CPU.

In scope now: the two mice the user owns, both connected through their 2.4 GHz dongles.

| Mouse | Dongle (VID:PID) | Verified reading |
|---|---|---|
| Ajazz (firmware tag `M184`) | `248a:5b2f` "SSCYPL Wireless-Receiver" | 77 % |
| Keychron M6 (mouse PID `d03f`) | `3434:d031` "Keychron Link" (Type-C) | 76–78 % |

Out of scope for now: Bluetooth (GNOME already reads it from BlueZ through UPower), other mice,
changing any mouse setting.

Success means the top bar shows a correct percentage for each connected mouse, the daemon stays under
about 2 MB of RSS, and it uses no CPU while idle.

## Protocols (verified on this hardware on 2026-11-20)

Both protocols are read-only. Nothing is ever written to the mouse's settings.

### Ajazz: poll

- Interface 1 of `248a:5b2f`, vendor usage page `0xFF00`, feature report `0x0C`, 33 bytes including
  the report ID.
- Request, sent with `HIDIOCSFEATURE(33)`: `0c 01 20 00 <seq> 00 00`, zero padded.
- After about 30 ms, `HIDIOCGFEATURE(33)` with byte 0 set to `0x0c` returns the reply:
  - `rx[2] == 0x20`, and `rx[8..12]` is an ASCII tag (`M184`).
  - `rx[0x12]` is the battery percentage, 0–100.
  - `rx[0x13]` is the charge state: 1 = charging, 2 = full, anything else = discharging.
- If `rx[6] == 0`, the reply isn't ready yet, so retry once.
- Source: the Zircon 500 notes (same receiver); live reply from this mouse:
  `0c 01 20 00 01 01 10 00 4d 31 38 34 33 01 00 95 11 ff 4d 00 …`.

### Keychron: listen (the receiver pushes, it doesn't answer requests)

- Interface 1 of `3434:d031`, usage page `0x8C`. Input report `0x54` is 20 bytes.
- When the mouse connects or wakes, the receiver sends `54 e2 01 <link> 00 <pct> 02 02 00 …`.
  Seen on this hardware: `54 e2 01 02 00 4e …` straight after power-on, then
  `54 e2 01 01 00 4c …`.
  - `byte[1] == 0xE2` means a status event.
  - `byte[3]`: 01 = connected, 02 = seen once right after power-on.
  - `byte[5]` is the battery percentage, 0–100. Accept it only when `byte[6..8] == 02 02`; the
    zero-filled variant `54 e2 01 01 00 00 …` carries no reading.
  - Charging state is unknown for this report, so it is reported as `null`.
- Feature reads (`51 xx`) always fail with `EPIPE` on this receiver, on Linux and from Keychron's own
  Launcher, so the daemon never sends them.
- Evidence: a usbmon capture of Keychron Launcher in a Windows VM, plus a live capture on Linux.

## Architecture

```
  receivers (hidraw)                 whiskerd (Rust, systemd --user)
  ┌──────────────┐  poll every 5 min   ┌───────────────────────────────┐   atomic write
  │ Ajazz 5b2f   │◄────────────────────┤ drivers::ajazz                │
  └──────────────┘                     │ drivers::keychron             ├──► $XDG_RUNTIME_DIR/
  ┌──────────────┐  pushed 0x54 e2     │ discovery (sysfs + netlink)   │    whiskerd.json
  │ Keychron d031├────────────────────►│ one poll(2) loop, no threads  │          │ inotify
  └──────────────┘                     └───────────────────────────────┘          ▼
                                                               GNOME Shell extension (panel items)
```

### Daemon: `whiskerd` (Rust)

- Dependencies: `libc` only. No async runtime and no serde; the JSON is written by hand.
- Event loop: a single `poll(2)` over
  - a netlink `NETLINK_KOBJECT_UEVENT` socket, which triggers a rescan when a `hidraw` device is added
    or removed, so a dongle plugged in later shows up at once;
  - each open Keychron hidraw fd, to receive the pushed `0x54` reports;
  - a timeout set to the next Ajazz poll (300 s).
- Modules, each with one job:
  - `hidraw`: open nodes, plus `HIDIOCSFEATURE`/`HIDIOCGFEATURE` ioctl wrappers.
  - `discovery`: scan `/sys/class/hidraw` and match on `HID_ID` and the parent `bInterfaceNumber`.
  - `drivers/ajazz.rs` and `drivers/keychron.rs`: a pure `parse(&[u8]) -> Option<Reading>`, plus
    the I/O around it.
  - `state`: the device list, writing the JSON when something changes (temp file + `rename`).
- A device whose node disappears is removed. A mouse that stops answering keeps its last reading and
  timestamp, with `"connected": false`.
- Logs go to stderr and land in the journal. Normal operation is silent; only errors and devices
  appearing or disappearing are logged.

### Output file

`$XDG_RUNTIME_DIR/whiskerd.json` lives on tmpfs, so there are no disk writes:

```json
{"devices":[
  {"id":"ajazz-248a5b2f","name":"Ajazz M184","battery":77,"charging":false,"connected":true,"updated":1795188749},
  {"id":"keychron-3434d031","name":"Keychron M6","battery":76,"charging":null,"connected":true,"updated":1795188750}
]}
```

`battery` is `null` until the first reading arrives. For Keychron that happens the first time the
mouse wakes after the daemon starts.

### GNOME Shell extension: `whiskerd@zamiul-rashid`

- A GNOME 45+ ESM extension for GNOME 50.
- A single `PanelMenu.Button` shows one `input-mouse-symbolic` icon plus `NN%` for each device that
  has a reading.
- At 20 % or below, the label uses the warning colour. While charging, a ⚡ is shown.
- The dropdown lists each mouse with its percentage and "updated N min ago".
- It watches the JSON file with `Gio.FileMonitor` (inotify) and never polls. If the file is missing,
  it hides itself.

### Permissions and install

- `/etc/udev/rules.d/70-whiskerd.rules`: `SUBSYSTEM=="hidraw"`, matching each dongle's
  VID:PID, `TAG+="uaccess"`. The logged-in user gets access and nothing runs as root.
- `~/.config/systemd/user/whiskerd.service`:
  - `Restart=on-failure`;
  - `MemoryMax=16M`, as a guard rail;
  - `NoNewPrivileges=yes`.
- `install.sh` builds `--release`, copies the binary to `~/.local/bin`, installs the unit and the
  extension, and installs the udev rule with `sudo` (the only privileged step).

## Testing

- Unit tests for both `parse` functions, using the exact bytes captured on 2026-11-20. These include
  the zero-filled Keychron variant and an Ajazz reply that isn't ready yet.
- Unit tests for the JSON writer (escaping, `null`s) and for discovery against a fake sysfs tree in a
  temp directory.
- Manual end-to-end check on this laptop:
  - The daemon running shows both mice in the top bar.
  - Unplugging a dongle removes its item.
  - Switching the M6 off and on updates its value.
  - `ps -o rss` stays below 2 MB.

## Risks and open questions

- How often the Keychron receiver pushes `e2` while the mouse stays awake is unknown; only
  connect/wake events have been seen. Showing the last value with its age makes a stale reading
  visible.
- Whether the Ajazz receiver answers while its mouse is asleep is unknown. If it doesn't, the last
  value is kept.

## Addendum: what shipped in v0.1.0 (2026-11-20)

These changes were agreed with the user while building and supersede the sections above where they
differ.

- **Daemon, not a timer.** Keychron receivers only push their battery when the mouse wakes, so
  `whiskerd` runs as a long-lived systemd user service. It blocks in one `poll(2)` and its idle cost
  is 0 % CPU and about 2 MB RSS.
- **Protocol families plus a receiver table** (`src/devices.rs`). Each entry is marked *verified* or
  *community-reported*. The udev rule is generated from the table with `whiskerd --udev-rules`.
- **Kernel peripherals.** `/sys/class/power_supply` entries with scope `Device` are included (for
  example Logitech through hid-logitech-hidpp). JSON entries gained a `kind` field.
- **Sleeping Compx mice.** When the mouse sleeps, the receiver answers with zeros, so the daemon
  retries every 30 s until it gets a reading, then every 300 s. Keychron receivers whose setup failed
  (ACL not applied yet, interface 3 late) are retried on the same schedule and on every hidraw event.
- **Hotplug** uses udev's netlink group (2), not the kernel group, so the `uaccess` ACL is already in
  place when the daemon hears about a node.
- **Extra CLI:**
  - `--output -` prints JSON lines for other bars;
  - `--probe` and `--list` are for diagnostics.
- **Distribution:**
  - static musl binaries for x86_64 and aarch64;
  - nfpm-built `.deb`, `.rpm` and Arch packages;
  - a signed apt repo and a signed dnf repo on GitHub Pages;
  - a PKGBUILD for the AUR;
  - `install.sh` for source installs.

  Signing happens in a separate CI job that holds the key and runs no third-party actions.
