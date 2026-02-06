# Changelog

## 0.6.2

- Fix off-by-one at formatting boundaries where the first character of an insertion was misclassified as part of the preceding deletion

## 0.6.1

- Redline `type` field now returns atoms (`:insertion`, `:deletion`, `:paired`) instead of strings
- Remove eager loading of Python test support module from test helper
- Add Rust version to `.tool-versions`
- Add development build instructions to README

## 0.6.0

Major accuracy improvements to the redline extraction algorithm. On a sample of 50 redline PDFs, capture rate improved from 53.5% to 98.4%.

### Text extraction overhaul

- Replaced device-level span boundaries with MuPDF's structured text (stext) line/style grouping, matching the same text model PyMuPDF uses internally
- Synthesize space characters by glyph geometry, fixing missing spaces throughout extracted text
- Adaptive intervening-text break thresholds to correctly split segments separated by uncolored content (email/token mode at 2.3x, punctuation at 2.5x, name boundaries at 3.2x, prose at 5.0x)

### Pairing improvements

- Fixed x-gap calculation to measure from segment end (not start), preventing false pairings
- Tightened pair_x_gap_max from 3.0 to 1.5 points to avoid pairing adjacent but unrelated items
- Sort deletions first to match Python pairing order
- Allow overlapping deletion/insertion positions

### Color handling

- Use MuPDF's ICC-aware color conversion (`Colorspace::convert_color`) instead of naive CMYK-to-RGB formula, fixing missed redlines in CMYK documents

### Segment boundary fixes

- Strip font subset prefixes (e.g. `UFLVUZ+`) in style key to prevent fragmentation across font subsets
- Flush segments on backward x-jumps to handle overlaid duplicate text layers
- Add name boundary break heuristic for table layouts with adjacent names
- Add comma to punctuation break list

## 0.5.0

- Initial release
