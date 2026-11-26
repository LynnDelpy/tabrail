# The 15 new palette colours

Appended to `palette` after today's 25, in this order (New group… takes the first one no group uses, so the most distinct come first). Each line in `palette` reads `#rrggbb Name`.

| Hex | Name | OKLab L | Chroma | Hue° | Contrast to `#cdd6f4` | Nearest palette colour, distance |
|---|---|---|---|---|---|---|
| `#4f4821` | Mustard | 0.40 | 0.058 | 99 | 6.4 | Bronze 4.3 |
| `#5d372a` | Rust | 0.38 | 0.059 | 40 | 7.1 | Brick 4.4 |
| `#1d5249` | Jade | 0.40 | 0.058 | 180 | 6.2 | Fern 4.5 |
| `#303008` | Moss | 0.30 | 0.057 | 109 | 9.4 | Umber 4.8 |
| `#5f5932` | Khaki | 0.46 | 0.057 | 101 | 4.9 | Copper 5.1 |
| `#023729` | Pine | 0.30 | 0.058 | 170 | 9.2 | Green 4.5 |
| `#424a40` | Sage | 0.40 | 0.019 | 139 | 6.4 | Fern 4.5 |
| `#084649` | Petrol | 0.36 | 0.058 | 200 | 7.3 | Green faint 4.5 |
| `#522b2b` | Brick | 0.34 | 0.058 | 21 | 8.3 | Rust 4.4 |
| `#3c5536` | Fern | 0.42 | 0.058 | 140 | 5.7 | Green deep 4.4 |
| `#43260b` | Umber | 0.30 | 0.059 | 60 | 9.5 | Brown 4.3 |
| `#734e3a` | Copper | 0.46 | 0.059 | 49 | 5.0 | Bronze 4.2 |
| `#734a57` | Berry | 0.46 | 0.059 | 359 | 5.1 | Pink deep 4.2 |
| `#334321` | Forest | 0.36 | 0.058 | 130 | 7.4 | Olive 4.2 |
| `#64452b` | Bronze | 0.42 | 0.058 | 60 | 6.0 | Peach deep 4.1 |

Distance is Euclidean OKLab × 100 over all 40 colours. For comparison, the closest pairs among today's 25 are 1.8 apart (Green faint / Teal faint, Mauve faint / Blue faint).

## How they were chosen

- Candidates: OKLCH L 0.30–0.46 in steps of 0.02, chroma 0.02, 0.03, 0.04, 0.05 and 0.058, hue every 10°, inside sRGB, bold `#cdd6f4` text at 4.5:1 or more. This is the same lightness and chroma range as today's 25 (L 0.30–0.47, chroma at most 0.058), so the new tints are just as faint.
- Greedy: add the candidate farthest from every colour already in the palette, 15 times.
- Today's tints are Catppuccin accents blended onto `#1e1e2e`, which turns most of them grey-violet (hues 245–345). The new ones fill the oranges, olives, greens and teals the palette lacks. Blending the 14 accents at more depths reaches 40 only with new colours 2.5 apart.
- Names: one word each, unique, and no existing colour's name (`teal`, `rose`, `blue` …) fuzzy-matches them. So typing an old name in the picker still finds only its own rows.
