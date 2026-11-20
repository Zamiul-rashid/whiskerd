# Product

<!-- impeccable:product-schema 1 -->

## Platform

web

## Stack

Project website: a static site generator, chosen by the owner. Which generator was delegated: Zola
(a single Rust binary with no node_modules, which fits a Rust repository; content is Markdown plus
Tera templates). Hosted on GitHub Pages at https://zamiul-rashid.github.io/whiskerd/, on the same
`gh-pages` branch that serves the signed apt (`/apt`) and dnf (`/rpm`) repositories and the key
(`/whiskerd.asc`). The site replaces the old plain "package repositories" front page; those paths
must keep working.

## Users

Inferred, not confirmed by the owner: Linux desktop users (mostly GNOME, some Waybar/polybar/eww)
who own an inexpensive 2.4 GHz wireless mouse (Ajazz, Delux, Genesis, Keychron ...) and found that
Linux shows no battery level for it. Their two questions, in order: "does my mouse work?" and
"how do I install it?". Contributors who want to add a receiver are a secondary audience.

## Product Purpose

whiskerd is a tiny daemon that reads battery levels straight from wireless mouse USB receivers,
read-only, and publishes them as a small JSON file; a GNOME Shell extension shows them in the top
bar. Success: the top bar shows the correct percentage for each connected mouse, without vendor
software, without root at runtime.

## Positioning

It talks to the receiver itself, using reverse-engineered vendor protocols, for mice the kernel
cannot read. About 2 MB of RAM, no CPU while idle (one `poll()` loop, no threads, libc only),
hotplug-aware, and it never changes a mouse setting.

## Operating Context

Installed from signed apt/dnf repositories, an Arch package, or from source with `install.sh`.
Runs as a systemd user service. Output at `$XDG_RUNTIME_DIR/whiskerd.json`; `whiskerd --output -`
streams JSON lines for other bars. `whiskerd --probe` and `--list` are the diagnostics.

## Capabilities and Constraints

- Linux only. Windows is not supported (a port is possible but not started).
- Supported receivers and the evidence per mouse live in `docs/compatible-mice.md` and
  `src/devices.rs`. Evidence levels: tested, same protocol tested elsewhere, expected. Never upgrade
  a mouse's level without evidence.
- Keychron battery appears only when the mouse connects or wakes.

## Brand Commitments

- Name: `whiskerd`, lowercase.
- Mascot icon chosen by the owner: Flaticon "mouse" icon 1682033 (flat cartoon mouse: grey body,
  pink ears, tail and feet), used in the GNOME top bar. Flaticon's free license requires
  attribution.
- Voice: plain, factual, specific; the README's tone.

## Evidence on Hand

- Real JSON output sample and real protocol bytes (README, `docs/superpowers/specs/`).
- Tested hardware: Ajazz AJ179P (reports firmware tag M184), Keychron M6.
- No screenshots of the GNOME top bar exist yet. No users, testimonials, download counts or
  benchmarks beyond the stated 2 MB RSS target; do not invent any.

## Product Principles

1. Honesty about compatibility beats a long list: every mouse carries its evidence level.
2. Small and quiet: no resident cost, no settings changed, no root.
3. Answer "does my mouse work?" before anything else.
4. Copy-pasteable install for each distro.
