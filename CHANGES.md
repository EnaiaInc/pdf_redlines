# Cross-Compilation Fix Summary

## Problem

Cross-compilation for aarch64-unknown-linux-gnu was failing because MuPDF's embedded font object files (.cff.o) were being compiled for x86_64 instead of aarch64, causing linker errors.

## Solution

Disabled embedded fonts entirely using TOFU (TOfu FOr Unicode) flags:

- `CFLAGS="-DTOFU -DTOFU_CJK_LANG -DTOFU_CJK_EXT"`
- Applied globally to all platforms for consistency

## Changes Made

### 1. Cross.toml (aarch64-linux only)

- Added TOFU CFLAGS to env section
- Kept make wrapper (doesn't hurt, might help other builds)

### 2. release.yml (all platforms)

- Added TOFU CFLAGS as global env var
- Applies to macOS, Linux x86_64, and Linux aarch64

### 3. Cargo.toml

- Stripped unnecessary features from mupdf dependency
- Before: `["js", "xps", "svg", "cbz", "img", "html", "epub", "system-fonts"]`
- After: `["system-fonts"]`
- We only need PDF parsing, not rendering of other formats

## Benefits

### Binary Size

- **Before:** ~50MB per platform
- **After:** Expected <10MB per platform
- Embedded CJK fonts were the bulk of the size

### Build Time

- Fewer features to compile
- Faster cross-compilation

### Consistency

- Same behavior across all platforms
- No embedded fonts anywhere

### Security

- Smaller attack surface
- Removed unnecessary format parsers (JS, XPS, etc.)

## Why This Works for Redline Detection

Embedded fonts are for **rendering** PDFs visually. Our use case only needs:

1. Parse PDF structure
2. Read character positions (x, y, width, height)
3. Read character colors (RGB values)
4. Detect red text patterns

None of this requires font rendering or embedded fonts.

## Verification Needed

The Rust code has a fallback for character width:

```rust
char_width = font_size * 0.6 * scale
```

With system fonts, verify that:

1. Character width estimation remains accurate
2. Paired redline detection (x_gap, y_diff) still works
3. Multi-line merging doesn't break

Test with real PDFs on all platforms after CI passes.

## Next CI Run

All 9 jobs (3 platforms × 3 NIF versions) should now:

1. ✅ Build successfully
2. ✅ Create smaller binaries (~10MB each)
3. ✅ Complete faster

Monitor: https://github.com/EnaiaInc/pdf_redlines/actions
