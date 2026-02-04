# PDFRedlines

Fast PDF redline detection and extraction via a Rust NIF (MuPDF).

## Usage

```elixir
{:ok, result} = PDFRedlines.extract_redlines("/path/to/document.pdf")
# %PDFRedlines.Result{redlines: [%PDFRedlines.Redline{...}, ...]}

{:ok, true} = PDFRedlines.has_redlines?("/path/to/document.pdf")
```

## What Are Redlines?

Redlines are tracked changes embedded in PDFs, typically represented as:

- **Deletions**: colored text with a strikethrough line through the middle.
- **Insertions**: colored text with an underline below the text.

This library detects those visual signals and converts them into structured
entries (deletion, insertion, or paired change).

## Notes

- This library is intentionally small and focused on the minimal API we need.
- Precompiled NIFs are published in GitHub Releases; set `PDF_REDLINES_BUILD=1`
  to force a local build.

## Configuration

You can pass a keyword list or map to tune detection thresholds:

- `:red_r_min`
- `:red_g_max`
- `:red_b_max`
- `:blue_r_max`
- `:blue_g_max`
- `:blue_b_min`
- `:formatting_bar_height_max`
- `:formatting_bar_width_min`
- `:line_bar_height_max`
- `:line_bar_width_min`
- `:stroke_line_y_tolerance`
- `:stroke_line_width_min`
- `:line_break_height_ratio`
- `:same_line_y_tolerance`
- `:merge_x_gap_max`
- `:merge_line_height_min_ratio`
- `:merge_line_height_max_ratio`
- `:margin_end_ratio`
- `:margin_start_ratio`
- `:pair_x_gap_max`
- `:page_width_fallback`
- `:line_height_fallback`

### Tuning Guide (Quick)

- If **strikethroughs are missed**, try increasing:
  - `:formatting_bar_height_max`
  - `:stroke_line_width_min` (if lines are thicker)
- If **underlines are missed**, try increasing:
  - `:line_bar_height_max`
  - `:line_bar_width_min`
- If **colors are missed**, widen:
  - `:red_r_min` (lower for lighter reds)
  - `:blue_b_min` (lower for lighter blues)
- If **line wrapping isn’t merged**, adjust:
  - `:merge_line_height_min_ratio`, `:merge_line_height_max_ratio`
  - `:margin_start_ratio`, `:margin_end_ratio`

## Parity Test (Optional)

There is an optional parity test that compares Rust/MuPDF results against the
Python/PyMuPDF implementation. It is skipped by default.

Run it with:

```bash
TEST_PDF_REDLINES_PARITY=true mix test test/redlines_parity_test.exs
```

Inputs are read from `PDF_REDLINES_TEST_DIR` (defaults to `test/fixtures/pdfs`).

## Benchmarks

Run a basic benchmark across a folder of PDFs:

```bash
PDF_REDLINES_BUILD=1 mix pdf_redlines.bench
```

You can customize:

- `PDF_REDLINES_TEST_DIR` (default `test/fixtures/pdfs`)
- `PDF_REDLINES_BENCH_REPEATS` (default `3`)

## License

MIT
