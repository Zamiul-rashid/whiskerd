---
name: whiskerd
description: A sticker sheet. Die-cut stickers and label-maker strips on a blue-grey backing sheet.
colors:
  sheet: "#b9c5d3"
  sheet-deep: "#7b8da2"
  paper: "#ffffff"
  ink: "#1d2733"
  ink-soft: "#34414f"
  tape: "#2e3a48"
  pink: "#ff9cb3"
  panel: "#14181e"
typography:
  display:
    fontFamily: "Mochiy Pop One, Zen Maru Gothic, system-ui, sans-serif"
    fontSize: "clamp(2.25rem, 4.3vw, 3.7rem)"
    fontWeight: 400
    lineHeight: 1.12
    letterSpacing: "-0.01em"
  headline:
    fontFamily: "Mochiy Pop One, Zen Maru Gothic, system-ui, sans-serif"
    fontSize: "clamp(1.9rem, 3.4vw, 2.8rem)"
    fontWeight: 400
    lineHeight: 1.12
    letterSpacing: "-0.01em"
  title:
    fontFamily: "Zen Maru Gothic, system-ui, sans-serif"
    fontSize: "1.35rem"
    fontWeight: 900
    lineHeight: 1.25
  sticker-name:
    fontFamily: "Mochiy Pop One, Zen Maru Gothic, system-ui, sans-serif"
    fontSize: "1.15rem"
    fontWeight: 400
    lineHeight: 1.25
    letterSpacing: "-0.005em"
  body:
    fontFamily: "Zen Maru Gothic, system-ui, sans-serif"
    fontSize: "1.0625rem"
    fontWeight: 500
    lineHeight: 1.65
  label:
    fontFamily: "Zen Maru Gothic, system-ui, sans-serif"
    fontSize: "0.8rem"
    fontWeight: 900
    letterSpacing: "0.02em"
  mono:
    fontFamily: "ui-monospace, JetBrains Mono, Cascadia Mono, DejaVu Sans Mono, monospace"
    fontSize: "0.9em"
    fontWeight: 600
rounded:
  chip: "5px"
  inline: "6px"
  strip: "14px"
  sticker: "18px"
  pill: "999px"
spacing:
  gutter: "clamp(1rem, 4vw, 3rem)"
  sheet: "clamp(1.5rem, 4vw, 3rem)"
  section: "clamp(2.5rem, 6vw, 4.5rem)"
  grid-gap: "1.5rem 1.75rem"
components:
  button-copy:
    backgroundColor: "{colors.ink}"
    textColor: "{colors.paper}"
    typography: "{typography.label}"
    rounded: "{rounded.pill}"
    padding: "0.6rem 1.05rem"
  button-copy-hover:
    backgroundColor: "{colors.tape}"
  command-strip:
    backgroundColor: "{colors.paper}"
    textColor: "{colors.ink}"
    rounded: "{rounded.strip}"
    padding: "0.6rem 0.6rem 0.6rem 1.1rem"
  sticker-label:
    backgroundColor: "{colors.paper}"
    textColor: "{colors.ink}"
    rounded: "{rounded.sticker}"
    padding: "2.6rem 1.2rem 1.15rem"
  badge-tested:
    backgroundColor: "{colors.ink}"
    textColor: "{colors.paper}"
    typography: "{typography.label}"
    rounded: "{rounded.pill}"
    padding: "0.15rem 0.6rem"
  badge-elsewhere:
    backgroundColor: "{colors.sheet}"
    textColor: "{colors.ink}"
    typography: "{typography.label}"
    rounded: "{rounded.pill}"
    padding: "0.15rem 0.6rem"
  dieline:
    rounded: "{rounded.strip}"
    padding: "0.55rem 1rem 0.6rem"
  fact-sticker:
    backgroundColor: "{colors.paper}"
    textColor: "{colors.ink}"
    rounded: "{rounded.pill}"
    padding: "0.55rem 1.1rem"
  byte-tape:
    backgroundColor: "{colors.tape}"
    textColor: "#e9edf2"
    typography: "{typography.mono}"
    padding: "0.85rem 2rem"
  topbar-pill:
    backgroundColor: "{colors.panel}"
    textColor: "#eef1f5"
    rounded: "{rounded.pill}"
    padding: "0.6rem 1.1rem"
---

# Design System: whiskerd

## Overview

**Creative North Star: "The Sticker Sheet"**

The page is the backing sheet of a sticker sheet. Everything the visitor reads or takes away sits on something printed and cut: white kiss-cut stickers whose outlines follow the artwork, white label-maker strips holding commands, dark label tape carrying protocol bytes. The sheet itself is blue-grey, taken from the mascot's body, and below the hero it carries the release liner's faint printed lettering. Hairline crop marks register the corners of each printed area, and dashed perforations separate them.

Material, not decoration, carries meaning. How much is known about a mouse is shown by how far its sticker got through the print run: printed in full colour, printed in greyscale, or only a dashed die line with nothing printed yet. Stickers sit a degree or two off square, lift slightly off the sheet, and peel at one corner when touched. The tone is friendly and toylike in its type and shapes, and plain and factual in its words.

The world was chosen against the dark terminal-hero developer-tool page with a three-card feature grid.

**Key Characteristics:**
- Blue-grey backing sheet, white sticker stock, near-black ink; pink only where the mascot or a battery number is.
- Rounded Japanese pop display face over a rounded gothic body; a real monospace for bytes and commands.
- Soft drop-shadow lift that follows the cut shape, never a box.
- Small, alternating rotations; nothing perfectly square except the text.
- Evidence encoded as print state: full colour, greyscale, unprinted die line.

## Colors

A cool, low-chroma sheet-and-ink palette lifted from the mascot, with one warm accent held in reserve.

### Primary
- **Mascot Nose Ink** (ink): All text, link underlines (at 40% alpha), focus outlines, the Copy button, the Tested badge. Crop marks and perforations are this ink at 55% and 28% alpha.

### Secondary
- **Ear Pink** (pink): Battery percentages only: the top-bar "74%", the battery byte in a tape, the battery value in the JSON sample, plus the mascot's own ears, tail and feet in its artwork. Ink text on pink when it highlights a value.

### Neutral
- **Backing Sheet** (sheet): The page ground and the Tested-elsewhere badge. Also the browser theme colour.
- **Shaded Side** (sheet-deep): The mascot's shadowed tone; scrollbar thumb, inner frame of the Tested-elsewhere legend chip. Not a text background: white on it measures 3.4:1.
- **Sticker Stock** (paper): Every sticker, strip, fact label and code panel; the kiss-cut border.
- **Soft Ink** (ink-soft): Secondary prose, notes, captions, the lede.
- **Label Tape** (tape): Byte tapes, text selection, Copy button hover.
- **Top Bar** (panel): Only the GNOME top-bar sticker, which reproduces the real panel.

### Named Rules
**The Battery Pink Rule.** Pink marks a battery level or belongs to the mascot. It is never a button, link, border or heading colour. If a pink element is not a battery number, it is wrong.

**The Mascot Palette Rule.** New neutrals come from the mascot's body tones (sheet, sheet-deep) or ink alphas, not from a new hue.

## Typography

**Display Font:** Mochiy Pop One 400 (with Zen Maru Gothic, system-ui)
**Body Font:** Zen Maru Gothic 500/700/900 (with system-ui)
**Label/Mono Font:** system monospace stack (ui-monospace, JetBrains Mono, Cascadia Mono, DejaVu Sans Mono)

**Character:** A wide, bubbly Japanese pop face for names and headlines over a rounded gothic that stays calm at reading sizes. All faces are self-hosted woff2 with `font-display: swap`; the display face is preloaded.

### Hierarchy
- **Display** (400, clamp(2.25rem, 4.3vw, 3.7rem), 1.12): The hero headline only, balanced, two to three lines on desktop.
- **Headline** (400, clamp(1.9rem, 3.4vw, 2.8rem), 1.12): Section headings.
- **Title** (Zen Maru Gothic 900, 1.2 to 1.35rem, 1.25): Receiver group and install-distro headings.
- **Sticker name** (Mochiy Pop One 400, 1.15rem; 1rem on die lines): Mouse names on stickers and die lines, and the wordmark (1.35rem).
- **Body** (500, 1.0625rem, 1.65): Prose, held to 34 to 46rem measures. Secondary text steps down to 0.9 to 0.97rem in soft ink.
- **Label** (900, 0.8rem, 0.02em): Evidence badges. Facts, nav and Copy use 700.
- **Mono** (600 on strips and tapes): Commands, byte values, IDs, JSON.

### Named Rules
**The Two Voices Rule.** The pop face names things (headings, mice, the wordmark) at weight 400 only; everything that explains is Zen Maru Gothic. Never set prose in Mochiy Pop One.

**The Unbroken Command Rule.** Inline code never wraps: it scrolls in place. Commands are copied, so they must stay intact.

## Layout

A single centred column, max 72rem, with a fluid side gutter (clamp(1rem, 4vw, 3rem)). The page is a stack of printed areas: hero, facts row, mice, how-it-works, install, footer. Printed areas carry crop marks in their four corners and inner padding of clamp(1.5rem, 4vw, 3rem); consecutive areas are split by a 2px dashed perforation and a clamp(2.5rem, 6vw, 4.5rem) gap.

The hero is a two-column grid (1.35fr / 1fr, about 57/43), headline and install strip left, mascot right. Below 52rem it becomes one column with the mascot first. Each receiver group is a 1fr / 1.75fr grid from 64rem up (group facts left, stickers right) and stacks below that. Stickers fill an auto-fill grid with a min(100%, 17rem) track; expected die lines wrap as a flex row of smaller outlines under them.

The release-liner lettering (a 340 x 120px tile) is a full-bleed layer behind the body, masked to transparent through the hero and fading in from 34rem to 52rem (62rem to 80rem on narrow screens), so the headline sits on clean sheet and no phrase is ever cut at an edge.

## Elevation & Depth

Depth is physical: paper lifted off a sheet. Shadows are soft, ink-tinted, two-layer drop-shadow filters, so they follow the real cut outline (mascot, kiss-cut border, notched tape) instead of a bounding box. Things on the sheet rest at the low lift; the mascot, the top-bar sticker and a touched sticker rise to the high lift. Die lines have no lift because nothing is printed on them.

### Shadow Vocabulary
- **Lift** (`filter: drop-shadow(0 1px 1px rgba(29,39,51,.22)) drop-shadow(0 8px 14px rgba(29,39,51,.16))`): Strips, fact stickers, mouse stickers at rest, the JSON panel.
- **High lift** (`filter: drop-shadow(0 2px 2px rgba(29,39,51,.2)) drop-shadow(0 18px 24px rgba(29,39,51,.22))`): Hero mascot, top-bar sticker, a sticker being peeled.
- **Kiss-cut border** (eight 2 to 3px white drop-shadows on the sticker wrapper): Traces a white border around the mascot-plus-label alpha. Applied before the lift.
- **Tape** (`drop-shadow(0 3px 4px rgba(29,39,51,.25))`): Byte tapes only.

### Named Rules
**The Follow-The-Cut Rule.** Anything with a cut outline (mascot, kiss-cut sticker, tape, strip) lifts with `filter: drop-shadow`, so the shadow traces the cut shape. `box-shadow` is reserved for rings (the top-bar sticker's 7px white kiss-cut ring, the Copy failure ring) and the tiny rectangular legend chips.

## Shapes

Everything is rounded and slightly tilted. Corners scale with the object: 5px for tiny chips and byte cells, 6px for inline code, 14px for strips and die lines, 18px for sticker labels, full pills for buttons, fact labels, badges and the top-bar sticker. Fact labels alternate -1.5 and 1.2 degrees; mouse stickers cycle -1.4, 1, -0.5, 1.5 degrees; the mascot sits at -5 and the top-bar sticker at 4. Byte tapes have notched ends (12px V cuts). Unprinted things use a 2px dashed ink outline. The only straight lines are crop marks, perforations and one hairline above the missing-mouse note.

## Components

### Buttons
Copy is the only button: a dark ink pill that sits at the end of a command strip.
- **Shape:** full pill (999px).
- **Default:** ink fill, white Zen Maru Gothic 700 at 0.9rem, 0.6rem 1.05rem padding.
- **Hover / Active:** tape fill; presses to scale(0.96). Transitions 0.2s on the house ease.
- **Copied:** label "Copied" in ink on the sheet fill, reverts after 2s. A confirmation state keeps at least 4.5:1.
- **Clipboard refused:** white with a 2px ink ring, label "Press Ctrl+C", and the command is selected for manual copy.

### Command Strip (label-maker strip)
- **Style:** white paper, 14px radius, low lift, mono 600 command filling the strip, Copy at the end.
- **Block variant:** multi-line `pre` at 0.88rem / 1.7, button aligned to the top.

### Mouse Sticker (signature)
A die-cut sticker: the mascot icon (72px) breaks the top edge of a white 18px-radius label holding the mouse name in the display face, an evidence badge and a note. A white kiss-cut border traces both together, then the lift.
- **Tested:** full-colour mascot, ink badge with white text.
- **Tested elsewhere:** greyscale mascot, sheet-coloured badge with ink text.
- **Expected:** not a sticker. A dashed 2px ink outline (50% alpha) on an 18%-white wash, 14px radius, name and note only, no mascot, no lift.
- **Peel (hover or focus-within, hover-capable devices):** rotation straightens, rises 3px to the high lift, the bottom-right 30px corner is clipped off and replaced by a curled silver-grey triangle. 0.35s on the house ease.

### Legend Chips
Small 1.6 x 1.1rem swatches that repeat the three print states: white with an ink inner frame, white with a sheet-deep inner frame, and a dashed outline.

### Fact Stickers
White pill labels, Zen Maru Gothic 700, low lift, alternating tilt, centred in a wrapping row.

### Byte Tape
Dark label-maker tape with notched ends: mono 600 bytes in near-white, the battery byte on pink with ink text, tagged bytes on a 16% white wash. A decode line beneath repeats the highlight as inline pink and white chips.

### Top-bar Sticker
A reproduction of the GNOME panel as a dark pill (panel colour) with a 7px white kiss-cut ring holding only what whiskerd adds to the bar: the mascot icon and the pink percentage. It overlaps the mascot's lower left at 4 degrees. On load it settles once from above, larger and more tilted, onto the sheet (0.9s, 0.25s delay).

### Navigation
Masthead: wordmark (28px mascot icon plus "whiskerd" in the display face) left, Zen Maru Gothic 700 text links right, no underline until hover. Wraps on narrow screens.

### Motion
One ease for everything: cubic-bezier(0.16, 1, 0.3, 1). Under `prefers-reduced-motion` every animation and transition is removed and smooth scrolling is off; the page is static.

## Do's and Don'ts

### Do:
- **Do** put content on a printed object (sticker, strip, tape, label) and leave the sheet itself for headings and prose.
- **Do** encode evidence as print state: full colour for tested, greyscale for tested elsewhere, an unprinted dashed die line for expected.
- **Do** lift with two-layer ink-tinted `drop-shadow` filters (the Lift and High lift values) so shadows follow the cut.
- **Do** tilt stickers by one or two degrees and alternate the direction.
- **Do** keep pink for battery numbers and the mascot's own colours.
- **Do** register printed areas with hairline crop marks and separate them with a 2px dashed perforation.
- **Do** honour reduced motion with a fully static page.

### Don't:
- **Don't** build a dark terminal-hero developer-tool page with a three-card feature grid.
- **Don't** use pink for buttons, links, borders or headings.
- **Don't** set prose or labels in Mochiy Pop One, or use it at any weight but 400.
- **Don't** lift a cut shape with `box-shadow`, and don't use a hard offset shadow of any kind.
- **Don't** give an expected mouse a mascot, a lift or a fill; it has not been printed.
- **Don't** let a command wrap; let it scroll inside its strip.
