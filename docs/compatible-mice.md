# Compatible mice

whiskerd reads the battery from the mouse's USB receiver, not from the mouse. So what matters is
which receiver your mouse came with. Run `lsusb` to see its USB ID, then `whiskerd --probe` to see
whether whiskerd can read it.

Each mouse below has one of three levels of evidence:

- **Tested**: this project has read the battery from that mouse on real hardware.
- **Same protocol, tested elsewhere**: another open-source project reads this mouse with the same
  bytes whiskerd sends, but nobody has tried it with whiskerd yet.
- **Expected**: the mouse uses a receiver whiskerd supports, but nobody has confirmed the battery
  reading yet.

## Compx receivers (`248a:5b2f` "SSCYPL Wireless-Receiver")

A generic Telink/Compx 2.4 GHz receiver that several brands ship. whiskerd sends feature report
`0x0C` with command `0x20` and reads the battery percentage and charge state from the reply.

| Mouse | Evidence | Shown as | Notes |
|---|---|---|---|
| Ajazz AJ179P | Tested | Ajazz AJ179P | Reports firmware tag `M184`. Other Ajazz models may report the same tag. |
| Delux M800 Pro | Same protocol, tested elsewhere | `Mouse <tag>` | [Wireless-Device-Battery-Monitor #156](https://github.com/krisgfx9/Wireless-Device-Battery-Monitor/pull/156) reads it with the same query, battery at byte 18 and charge state at byte 19. The cable ID `248a:5b2e` is listed too. |
| Genesis Zircon 500 (v1) | Expected | Genesis Zircon 500 | Firmware tag `M137`. Uses the same receiver and wired IDs according to [zircon500-linux-cli](https://github.com/cukierbezsoli/zircon500-linux-cli). |
| Other mice with this receiver | Expected | `Mouse <tag>` | The receiver ID also appears with Maxxter and Langtu mice. Whether those use the same battery command is unknown. |

The same mice plugged in by cable show up as `248a:5b2e` or `331a:5024`. whiskerd supports those
IDs, but nobody has tested them with whiskerd yet.

A mouse it doesn't recognize appears as `Mouse <tag>`, where `<tag>` is the model tag the mouse
reports. To give it a proper name, add its tag to `model_name` in
[`src/drivers/compx.rs`](../src/drivers/compx.rs).

## Keychron Link receivers (`3434:d031` Type-C, `3434:d030` Type-A)

The receiver sends the battery level whenever the mouse connects or wakes up. Right after login,
the level appears the first time you use the mouse.

| Mouse | Evidence | Shown as |
|---|---|---|
| Keychron M6 | Tested (on `3434:d031`) | Keychron M6 |
| Keychron M1 | Expected | Keychron M1 |
| Keychron M2 | Expected | Keychron M2 |
| Keychron M2 mini | Expected | Keychron M2 mini |
| Keychron M3 | Expected | Keychron M3 |
| Keychron M3 mini | Expected | Keychron M3 mini |
| Keychron M7 | Expected | Keychron M7 |

The mouse names come from the paired mouse's product ID, which the receiver reports. Any other
Keychron mouse paired with a Link receiver still shows up, as "Keychron mouse".

Some Keychron mice come with a 4K or 8K receiver, not the Link. Those receivers have other USB IDs,
so whiskerd does not detect them yet. If you have one, please open an issue with your `lsusb` output.

## Mice the Linux kernel already reads

whiskerd also shows any peripheral battery the kernel reports in `/sys/class/power_supply` (scope
`Device`). The most common case:

- **Logitech** mice on Unifying, Bolt or Lightspeed receivers, read by the kernel's
  `hid-logitech-hidpp` driver.

To check whether the kernel already sees your mouse's battery, run `ls /sys/class/power_supply`.
A mouse with a kernel driver appears there, typically named after the driver (for example
`hidpp_battery_0`).

## Your mouse isn't listed?

Open an issue with the output of `lsusb` and `whiskerd --probe`. If the receiver uses a protocol
whiskerd already speaks, adding it is one line in [`src/devices.rs`](../src/devices.rs).
