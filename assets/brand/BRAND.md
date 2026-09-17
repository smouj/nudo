# NUDO — brand

The brand is small on purpose: a name, a symbol, two colours and a small set of
rules. This document says what exists, what may not be done with it, and what is
still missing.

## Name

| Context | Written | Example |
| ------- | ------- | ------- |
| The project, in prose and headings | `NUDO` | "NUDO is a programming language." |
| Ordinary sentences in documentation | `Nudo` | "Nudo's type system distinguishes…" |
| The toolchain, files and commands | `nudo` | `nudo check`, `nudo.toml`, `.nudo` |

No other variant is official. In particular:

* not `NuDo`
* not `N.U.D.O.`
* not `NUDO AI`
* not `NudoLang`
* not `nu`, and not `.nu` as a file extension — `.nu` is already associated with
  another ecosystem, and NUDO uses `.nudo`

The name is a noun. It is a language, so it takes a capital only at the start of
a sentence or in the brand spelling above.

## Meaning

A *nudo* is a knot. The metaphor is the project's thesis: separate strands —
people, code, agents, tools, models — held together by a structure rather than
by friction.

The symbol is an interlaced monogram in which the letters of the name can be
read. It is a single continuous form, with no separate parts, which is the point.

## Colours

| Role | Value | Notes |
| ---- | ----- | ----- |
| Primary (near-black) | `#000000` | The symbol's dominant form |
| Accent (blue) | `#3F85FF` | The interlaced strand in the delivered primary asset |
| Light background | `#FFFFFF` | Default page background |
| Dark background | `#000000` | Or any dark surface the mark can be read against |

The delivered assets use black and `#3F85FF`. The brand direction also calls for
a warm accent in the terracotta/orange family as the counterpoint to the near-black;
**no asset using that accent has been delivered**, so it is not in use. See
[Pending assets](#pending-assets).

What the brand deliberately does not use:

* purple-to-blue gradients, glowing networks, neurons, brains, chips, robots,
  hexagons, or circuit-board patterns;
* drop shadows, bevels, outer glows or 3D treatments;
* more than one accent colour at a time.

The identity should read as a language, not as an AI product.

## Logo variants

| Asset | Use | Status |
| ----- | --- | ------ |
| `assets/logo/nudo-symbol-primary.png` | The symbol, black with the blue interlace. Default on light backgrounds | Delivered |
| `assets/logo/nudo-symbol-light.png` | The symbol in black, single colour. Light backgrounds, print, favicons | Delivered |
| `assets/logo/nudo-symbol-dark.png` | The symbol in white, single colour. Dark backgrounds and dark mode | Delivered |
| `assets/logo/nudo-horizontal-dark.png` | Symbol plus wordmark, white. Dark backgrounds only | Delivered |
| `assets/logo/nudo-horizontal-primary.png` | Symbol plus wordmark, black and blue. Default lockup | **Pending** |
| `assets/logo/nudo-horizontal-light.png` | Symbol plus wordmark, black. Light backgrounds | **Pending** |

**The logo may not be redrawn, recoloured, re-proportioned or reconstructed.**
The three missing assets are missing; a substitute assembled from what exists
would be a different logo, and shipping one is worse than shipping none.

## Clear space

Reserve clear space around the mark equal to **25% of its height** on all sides.
Nothing enters that space: no text, no border, no other logo, no image edge.

## Minimum sizes

| Asset | Minimum | Why |
| ----- | ------- | --- |
| Symbol | 24 px | Below this the interlace stops reading as a knot |
| Horizontal lockup | 120 px wide | Below this the wordmark loses its letter spacing |

For anything smaller than 24 px, a single-colour symbol is the only permitted
form; the two-colour symbol loses its structure.

## Backgrounds

| Background | Permitted asset |
| ---------- | --------------- |
| White or light solid | `nudo-symbol-primary.png`, `nudo-symbol-light.png` |
| Black or near-black solid | `nudo-symbol-dark.png`, `nudo-horizontal-dark.png` |
| Photographic or busy imagery | None. Place the mark on a solid panel first |
| A colour | Only if contrast is at least 4.5:1 against the mark's darkest part |

## Incorrect use

* Do not rotate, skew, stretch or apply a perspective transform.
* Do not change the proportions of the symbol or the spacing of the wordmark.
* Do not apply gradients, shadows, glows, outlines or textures.
* Do not reorder or re-colour the strands.
* Do not place the two-colour symbol on a background where the blue or the black
  loses contrast.
* Do not add a tagline inside the lockup; it is not part of the mark.
* Do not use the symbol as a bullet, an icon font glyph, or a pattern.
* Do not combine the mark with another logo into a single lockup.

## Consistency

* Write `NUDO` in the repository, the documentation and the site. Write `nudo`
  when naming the binary, the manifest or a command.
* Use the primary lockup wherever a horizontal logo is possible; the symbol alone
  is for constrained spaces.
* Documentation pages use the symbol in a `<picture>` element so that light and
  dark mode both resolve, as
  [`../../README.md`](../../README.md) does.
* When a new asset is produced, it is added here with its status changed from
  *Pending* to *Delivered*. Nothing is added to `assets/logo/` without an entry.

## Pending assets

| Asset | Note |
| ----- | ---- |
| `nudo-horizontal-primary.png` | The default lockup. Required before any public announcement uses a horizontal mark |
| `nudo-horizontal-light.png` | Needed for light-mode documentation headers |
| Warm accent variant | The brand direction specifies a warm accent; no asset uses it yet |
| Favicon set | `.ico`/SVG favicons derive from the symbol once the horizontal assets exist |
| Social images | See [`../social/README.md`](../social/README.md) |

## Diagrams

Technical diagrams follow the palette above and live in
[`../diagrams/`](../diagrams/README.md). They are documentation, not brand
assets, and they may use additional neutral greys for structure.
