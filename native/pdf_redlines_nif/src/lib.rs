//! NIF for extracting redline (tracked changes) from PDF documents using MuPDF.
//!
//! This module provides Elixir bindings for the Rust-based redline extractor.

use mupdf::{
    ColorParams, Colorspace, Device, Document, Matrix, NativeDevice, Path, PathWalker, StrokeState,
    Text,
};
use rustler::Atom;
use rustler::{Encoder, Env, NifMap, NifResult, Term};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

// =============================================================================
// Detection Configuration
// =============================================================================

#[derive(Clone, Copy)]
struct Config {
    red_r_min: f32,
    red_g_max: f32,
    red_b_max: f32,
    blue_r_max: f32,
    blue_g_max: f32,
    blue_b_min: f32,
    formatting_bar_height_max: f32,
    formatting_bar_width_min: f32,
    line_bar_height_max: f32,
    line_bar_width_min: f32,
    stroke_line_y_tolerance: f32,
    stroke_line_width_min: f32,
    line_break_height_ratio: f32,
    same_line_y_tolerance: f32,
    merge_x_gap_max: f32,
    merge_line_height_min_ratio: f32,
    merge_line_height_max_ratio: f32,
    margin_end_ratio: f32,
    margin_start_ratio: f32,
    pair_x_gap_max: f32,
    page_width_fallback: f32,
    line_height_fallback: f32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            red_r_min: 0.5,
            red_g_max: 0.3,
            red_b_max: 0.4,
            blue_r_max: 0.3,
            blue_g_max: 0.6,
            blue_b_min: 0.5,
            formatting_bar_height_max: 2.0,
            formatting_bar_width_min: 3.0,
            line_bar_height_max: 2.0,
            line_bar_width_min: 3.0,
            stroke_line_y_tolerance: 2.0,
            stroke_line_width_min: 3.0,
            line_break_height_ratio: 0.5,
            same_line_y_tolerance: 2.0,
            merge_x_gap_max: 30.0,
            merge_line_height_min_ratio: 0.8,
            merge_line_height_max_ratio: 1.8,
            margin_end_ratio: 0.25,
            margin_start_ratio: 0.1,
            pair_x_gap_max: 5.0,
            page_width_fallback: 600.0,
            line_height_fallback: 15.0,
        }
    }
}

fn get_f32_from_map(map: &HashMap<Atom, Term>, key: Atom) -> Option<f32> {
    let value = map.get(&key)?;
    if let Ok(v) = value.decode::<f64>() {
        return Some(v as f32);
    }
    if let Ok(v) = value.decode::<i64>() {
        return Some(v as f32);
    }
    None
}

fn config_from_term(term: Term) -> Config {
    let mut config = Config::default();
    let Ok(map) = term.decode::<HashMap<Atom, Term>>() else {
        return config;
    };

    if let Some(v) = get_f32_from_map(&map, red_r_min()) {
        config.red_r_min = v;
    }
    if let Some(v) = get_f32_from_map(&map, red_g_max()) {
        config.red_g_max = v;
    }
    if let Some(v) = get_f32_from_map(&map, red_b_max()) {
        config.red_b_max = v;
    }
    if let Some(v) = get_f32_from_map(&map, blue_r_max()) {
        config.blue_r_max = v;
    }
    if let Some(v) = get_f32_from_map(&map, blue_g_max()) {
        config.blue_g_max = v;
    }
    if let Some(v) = get_f32_from_map(&map, blue_b_min()) {
        config.blue_b_min = v;
    }
    if let Some(v) = get_f32_from_map(&map, formatting_bar_height_max()) {
        config.formatting_bar_height_max = v;
    }
    if let Some(v) = get_f32_from_map(&map, formatting_bar_width_min()) {
        config.formatting_bar_width_min = v;
    }
    if let Some(v) = get_f32_from_map(&map, line_bar_height_max()) {
        config.line_bar_height_max = v;
    }
    if let Some(v) = get_f32_from_map(&map, line_bar_width_min()) {
        config.line_bar_width_min = v;
    }
    if let Some(v) = get_f32_from_map(&map, stroke_line_y_tolerance()) {
        config.stroke_line_y_tolerance = v;
    }
    if let Some(v) = get_f32_from_map(&map, stroke_line_width_min()) {
        config.stroke_line_width_min = v;
    }
    if let Some(v) = get_f32_from_map(&map, line_break_height_ratio()) {
        config.line_break_height_ratio = v;
    }
    if let Some(v) = get_f32_from_map(&map, same_line_y_tolerance()) {
        config.same_line_y_tolerance = v;
    }
    if let Some(v) = get_f32_from_map(&map, merge_x_gap_max()) {
        config.merge_x_gap_max = v;
    }
    if let Some(v) = get_f32_from_map(&map, merge_line_height_min_ratio()) {
        config.merge_line_height_min_ratio = v;
    }
    if let Some(v) = get_f32_from_map(&map, merge_line_height_max_ratio()) {
        config.merge_line_height_max_ratio = v;
    }
    if let Some(v) = get_f32_from_map(&map, margin_end_ratio()) {
        config.margin_end_ratio = v;
    }
    if let Some(v) = get_f32_from_map(&map, margin_start_ratio()) {
        config.margin_start_ratio = v;
    }
    if let Some(v) = get_f32_from_map(&map, pair_x_gap_max()) {
        config.pair_x_gap_max = v;
    }
    if let Some(v) = get_f32_from_map(&map, page_width_fallback()) {
        config.page_width_fallback = v;
    }
    if let Some(v) = get_f32_from_map(&map, line_height_fallback()) {
        config.line_height_fallback = v;
    }

    config
}

#[derive(Clone, Copy)]
#[allow(dead_code)]
struct PageMetrics {
    line_height: f32,
    page_width: f32,
}

// =============================================================================
// Rustler Setup
// =============================================================================

rustler::atoms! {
    ok,
    error,
    red_r_min,
    red_g_max,
    red_b_max,
    blue_r_max,
    blue_g_max,
    blue_b_min,
    formatting_bar_height_max,
    formatting_bar_width_min,
    line_bar_height_max,
    line_bar_width_min,
    stroke_line_y_tolerance,
    stroke_line_width_min,
    line_break_height_ratio,
    same_line_y_tolerance,
    merge_x_gap_max,
    merge_line_height_min_ratio,
    merge_line_height_max_ratio,
    margin_end_ratio,
    margin_start_ratio,
    pair_x_gap_max,
    page_width_fallback,
    line_height_fallback,
}

rustler::init!("Elixir.PDFRedlines.Native");

// =============================================================================
// NIF Result Types
// =============================================================================

#[derive(NifMap)]
struct NifRedline {
    r#type: String,
    deletion: Option<String>,
    insertion: Option<String>,
    location: String,
}

#[derive(NifMap)]
struct NifRedlineOutput {
    redlines: Vec<NifRedline>,
}

// =============================================================================
// Data Structures
// =============================================================================

/// A thin rectangle representing a strikethrough or underline formatting bar.
#[derive(Debug, Clone)]
struct FormattingBar {
    x1: f32,
    x2: f32,
    y: f32,
    #[allow(dead_code)]
    height: f32,
    page: usize,
}

/// A colored text character with its position.
#[derive(Debug, Clone)]
struct ColoredChar {
    char: char,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    /// Actual bbox top (y0), estimated from font metrics
    bbox_y0: f32,
    /// Actual bbox bottom (y1), estimated from font metrics
    bbox_y1: f32,
    /// Span identifier — segments are flushed at span boundaries (matching Python behavior)
    span_id: u64,
    page: usize,
    #[allow(dead_code)]
    y_flipped: bool,
}

/// A segment of text that is either all deletion or all insertion.
#[derive(Debug, Clone)]
struct TextSegment {
    text: String,
    is_deletion: bool,
    page: usize,
    y_pos: f32,
    x_pos: f32,
    x_end: f32,
}

// =============================================================================
// Color Detection
// =============================================================================

fn is_red_color(r: f32, g: f32, b: f32, config: Config) -> bool {
    r > config.red_r_min && g < config.red_g_max && b < config.red_b_max
}

fn is_blue_color(r: f32, g: f32, b: f32, config: Config) -> bool {
    r < config.blue_r_max && g < config.blue_g_max && b > config.blue_b_min
}

fn is_redline_color(r: f32, g: f32, b: f32, config: Config) -> bool {
    is_red_color(r, g, b, config) || is_blue_color(r, g, b, config)
}

fn extract_rgb(color: &[f32], colorspace: &Colorspace) -> Option<(f32, f32, f32)> {
    // Handle DeviceRGB (3 components)
    if colorspace.n() == 3 && color.len() >= 3 {
        return Some((color[0], color[1], color[2]));
    }
    // Handle DeviceCMYK - convert to RGB
    if colorspace.n() == 4 && color.len() >= 4 {
        let c = color[0];
        let m = color[1];
        let y = color[2];
        let k = color[3];
        let r = (1.0 - c) * (1.0 - k);
        let g = (1.0 - m) * (1.0 - k);
        let b = (1.0 - y) * (1.0 - k);
        return Some((r, g, b));
    }
    // DeviceGray - gray to RGB
    if colorspace.n() == 1 && !color.is_empty() {
        let gray = color[0];
        return Some((gray, gray, gray));
    }
    None
}

// =============================================================================
// Path Walking
// =============================================================================

struct RectExtractor {
    rects: Vec<(f32, f32, f32, f32)>,
    lines: Vec<(f32, f32, f32, f32)>,
    current_x: f32,
    current_y: f32,
    start_x: f32,
    start_y: f32,
}

impl RectExtractor {
    fn new() -> Self {
        Self {
            rects: Vec::new(),
            lines: Vec::new(),
            current_x: 0.0,
            current_y: 0.0,
            start_x: 0.0,
            start_y: 0.0,
        }
    }
}

impl PathWalker for RectExtractor {
    fn move_to(&mut self, x: f32, y: f32) {
        self.current_x = x;
        self.current_y = y;
        self.start_x = x;
        self.start_y = y;
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.lines.push((self.current_x, self.current_y, x, y));
        self.current_x = x;
        self.current_y = y;
    }

    fn curve_to(&mut self, _cx1: f32, _cy1: f32, _cx2: f32, _cy2: f32, ex: f32, ey: f32) {
        self.current_x = ex;
        self.current_y = ey;
    }

    fn close(&mut self) {
        self.current_x = self.start_x;
        self.current_y = self.start_y;
    }

    fn rect(&mut self, x1: f32, y1: f32, x2: f32, y2: f32) {
        self.rects.push((x1, y1, x2, y2));
    }
}

// =============================================================================
// Custom Device for Intercepting Drawing Operations
// =============================================================================

struct CollectorState {
    formatting_bars: Vec<FormattingBar>,
    colored_chars: Vec<ColoredChar>,
    current_page: usize,
    next_span_id: u64,
    config: Config,
}

struct RedlineCollector {
    state: Rc<RefCell<CollectorState>>,
}

impl RedlineCollector {
    fn new(state: Rc<RefCell<CollectorState>>) -> Self {
        Self { state }
    }
}

impl NativeDevice for RedlineCollector {
    fn fill_path(
        &mut self,
        path: &Path,
        _even_odd: bool,
        ctm: Matrix,
        color_space: &Colorspace,
        color: &[f32],
        _alpha: f32,
        _cp: ColorParams,
    ) {
        let Some((r, g, b)) = extract_rgb(color, color_space) else {
            return;
        };

        let config = self.state.borrow().config;
        if !is_redline_color(r, g, b, config) {
            return;
        }

        let mut extractor = RectExtractor::new();
        if path.walk(&mut extractor).is_err() {
            return;
        }

        let mut state = self.state.borrow_mut();
        let current_page = state.current_page;

        // Process rectangles
        for (x0, y0, x1, y1) in &extractor.rects {
            let tx0 = ctm.a * x0 + ctm.c * y0 + ctm.e;
            let ty0 = ctm.b * x0 + ctm.d * y0 + ctm.f;
            let tx1 = ctm.a * x1 + ctm.c * y1 + ctm.e;
            let ty1 = ctm.b * x1 + ctm.d * y1 + ctm.f;

            let (min_x, max_x) = if tx0 < tx1 { (tx0, tx1) } else { (tx1, tx0) };
            let (min_y, max_y) = if ty0 < ty1 { (ty0, ty1) } else { (ty1, ty0) };

            let width = max_x - min_x;
            let height = max_y - min_y;

            if height < config.formatting_bar_height_max && width > config.formatting_bar_width_min
            {
                state.formatting_bars.push(FormattingBar {
                    x1: min_x,
                    x2: max_x,
                    y: (min_y + max_y) / 2.0,
                    height,
                    page: current_page,
                });
            }
        }

        // Process line-based paths (filled polygons that form thin bars)
        if !extractor.lines.is_empty() {
            let mut path_min_x = f32::MAX;
            let mut path_max_x = f32::MIN;
            let mut path_min_y = f32::MAX;
            let mut path_max_y = f32::MIN;

            for (x0, y0, x1, y1) in &extractor.lines {
                let tx0 = ctm.a * x0 + ctm.c * y0 + ctm.e;
                let ty0 = ctm.b * x0 + ctm.d * y0 + ctm.f;
                let tx1 = ctm.a * x1 + ctm.c * y1 + ctm.e;
                let ty1 = ctm.b * x1 + ctm.d * y1 + ctm.f;

                path_min_x = path_min_x.min(tx0).min(tx1);
                path_max_x = path_max_x.max(tx0).max(tx1);
                path_min_y = path_min_y.min(ty0).min(ty1);
                path_max_y = path_max_y.max(ty0).max(ty1);
            }

            let path_width = path_max_x - path_min_x;
            let path_height = path_max_y - path_min_y;

            if path_height < config.line_bar_height_max && path_width > config.line_bar_width_min {
                state.formatting_bars.push(FormattingBar {
                    x1: path_min_x,
                    x2: path_max_x,
                    y: (path_min_y + path_max_y) / 2.0,
                    height: path_height,
                    page: current_page,
                });
            }
        }
    }

    fn stroke_path(
        &mut self,
        path: &Path,
        _stroke_state: &StrokeState,
        ctm: Matrix,
        color_space: &Colorspace,
        color: &[f32],
        _alpha: f32,
        _cp: ColorParams,
    ) {
        let Some((r, g, b)) = extract_rgb(color, color_space) else {
            return;
        };

        let config = self.state.borrow().config;
        if !is_redline_color(r, g, b, config) {
            return;
        }

        let mut extractor = RectExtractor::new();
        if path.walk(&mut extractor).is_err() {
            return;
        }

        let mut state = self.state.borrow_mut();
        let current_page = state.current_page;

        for (x0, y0, x1, y1) in extractor.lines {
            let tx0 = ctm.a * x0 + ctm.c * y0 + ctm.e;
            let ty0 = ctm.b * x0 + ctm.d * y0 + ctm.f;
            let tx1 = ctm.a * x1 + ctm.c * y1 + ctm.e;
            let ty1 = ctm.b * x1 + ctm.d * y1 + ctm.f;

            if (ty0 - ty1).abs() < config.stroke_line_y_tolerance {
                let width = (tx1 - tx0).abs();
                if width > config.stroke_line_width_min {
                    let min_x = tx0.min(tx1);
                    let max_x = tx0.max(tx1);
                    state.formatting_bars.push(FormattingBar {
                        x1: min_x,
                        x2: max_x,
                        y: (ty0 + ty1) / 2.0,
                        height: 1.0,
                        page: current_page,
                    });
                }
            }
        }
    }

    fn fill_text(
        &mut self,
        text: &Text,
        ctm: Matrix,
        color_space: &Colorspace,
        color: &[f32],
        _alpha: f32,
        _cp: ColorParams,
    ) {
        let Some((r, g, b)) = extract_rgb(color, color_space) else {
            return;
        };

        let config = self.state.borrow().config;
        if !is_redline_color(r, g, b, config) {
            return;
        }

        let mut state = self.state.borrow_mut();
        let current_page = state.current_page;

        for span in text.spans() {
            let font = span.font();
            let trm = span.trm();
            let font_ascender = font.ascender();
            let font_descender = font.descender();
            let span_id = state.next_span_id;
            state.next_span_id += 1;

            for item in span.items() {
                let ucs = item.ucs();
                if ucs <= 0 {
                    continue;
                }

                let Some(ch) = char::from_u32(ucs as u32) else {
                    continue;
                };

                let raw_x = item.x();
                let raw_y = item.y();

                let tx = ctm.a * raw_x + ctm.c * raw_y + ctm.e;
                let ty = ctm.b * raw_x + ctm.d * raw_y + ctm.f;

                let font_size = trm.d.abs().max(trm.a.abs());
                let scale = ctm.a.abs().max(ctm.d.abs());
                let char_width = font_size * 0.6 * scale;
                let char_height = font_size * scale;

                let glyph_id = item.gid();
                let width = if glyph_id >= 0 {
                    font.advance_glyph(glyph_id).unwrap_or(char_width / scale) * font_size * scale
                } else {
                    char_width
                };

                // Estimate bbox from font metrics (will be replaced by TextPage data if available)
                let bbox_y0 = ty - char_height * font_ascender;
                let bbox_y1 = ty - char_height * font_descender;

                state.colored_chars.push(ColoredChar {
                    char: ch,
                    x: tx,
                    y: ty,
                    width,
                    height: char_height,
                    bbox_y0,
                    bbox_y1,
                    span_id,
                    page: current_page,
                    y_flipped: ctm.d < 0.0,
                });
            }
        }
    }
}

// =============================================================================
// Formatting Detection
// =============================================================================

fn get_char_formatting(
    char_x: f32,
    char_width: f32,
    bbox_y0: f32,
    bbox_y1: f32,
    bars: &[FormattingBar],
    page: usize,
) -> Option<&'static str> {
    let char_x0 = char_x;
    let char_x1 = char_x + char_width;

    let y0 = bbox_y0;
    let y1 = bbox_y1;
    let text_height = y1 - y0;

    let strikethrough_zone_min = y0 + text_height * 0.2;
    let strikethrough_zone_max = y0 + text_height * 0.7;
    let underline_zone_min = y0 + text_height * 0.7;
    let underline_zone_max = y1 + text_height * 0.3;

    let mut has_strikethrough = false;
    let mut has_underline = false;

    for bar in bars {
        if bar.page != page {
            continue;
        }

        // Any x-overlap between bar and char (matching Python logic)
        if bar.x2 < char_x0 || bar.x1 > char_x1 {
            continue;
        }

        if strikethrough_zone_min <= bar.y && bar.y <= strikethrough_zone_max {
            has_strikethrough = true;
        } else if underline_zone_min <= bar.y && bar.y <= underline_zone_max {
            has_underline = true;
        }
    }

    if has_strikethrough {
        Some("strikethrough")
    } else if has_underline {
        Some("underline")
    } else {
        None
    }
}

// =============================================================================
// Text Segment Extraction
// =============================================================================

fn extract_text_segments(
    colored_chars: &[ColoredChar],
    formatting_bars: &[FormattingBar],
    _config: Config,
) -> Vec<TextSegment> {
    let mut segments = Vec::new();

    // Process chars in their natural order (preserving span structure from device callback).
    // Flush segments at span boundaries, matching Python's block→line→span→char iteration.
    let mut current_text = String::new();
    let mut current_formatting: Option<&str> = None;
    let mut current_span_id: u64 = u64::MAX;
    let mut current_page: usize = usize::MAX;
    let mut segment_y: f32 = 0.0;
    let mut segment_x: f32 = 0.0;
    let mut segment_x_end: f32 = 0.0;

    // Pre-collect bars by page for efficient lookup
    let mut bars_by_page: HashMap<usize, Vec<&FormattingBar>> = HashMap::new();
    for bar in formatting_bars {
        bars_by_page.entry(bar.page).or_default().push(bar);
    }

    for ch in colored_chars {
        let page = ch.page;
        let page_bars = bars_by_page.get(&page);
        let empty_bars: Vec<&FormattingBar> = Vec::new();
        let bars_ref = page_bars.unwrap_or(&empty_bars);
        let bars_owned: Vec<FormattingBar> = bars_ref.iter().map(|b| (*b).clone()).collect();

        // Flush segment on span boundary or page change
        if ch.span_id != current_span_id || ch.page != current_page {
            if !current_text.is_empty() && current_formatting.is_some() {
                let text = current_text.trim().to_string();
                if !text.is_empty() {
                    segments.push(TextSegment {
                        text,
                        is_deletion: current_formatting == Some("strikethrough"),
                        page: current_page,
                        y_pos: segment_y,
                        x_pos: segment_x,
                        x_end: segment_x_end,
                    });
                }
            }
            current_text.clear();
            current_formatting = None;
            current_span_id = ch.span_id;
            current_page = ch.page;
        }

        let formatting = get_char_formatting(
            ch.x,
            ch.width,
            ch.bbox_y0,
            ch.bbox_y1,
            &bars_owned,
            page,
        );

        if formatting.is_none() {
            if !current_text.is_empty() && current_formatting.is_some() {
                let text = current_text.trim().to_string();
                if !text.is_empty() {
                    segments.push(TextSegment {
                        text,
                        is_deletion: current_formatting == Some("strikethrough"),
                        page,
                        y_pos: segment_y,
                        x_pos: segment_x,
                        x_end: segment_x_end,
                    });
                }
            }
            current_text.clear();
            current_formatting = None;
            continue;
        }

        if current_formatting.is_none() {
            current_formatting = formatting;
            segment_y = ch.y;
            segment_x = ch.x;
            segment_x_end = ch.x + ch.width;
            current_text.push(ch.char);
        } else if formatting != current_formatting {
            // Formatting changed - flush segment
            let text = current_text.trim().to_string();
            if !text.is_empty() {
                segments.push(TextSegment {
                    text,
                    is_deletion: current_formatting == Some("strikethrough"),
                    page,
                    y_pos: segment_y,
                    x_pos: segment_x,
                    x_end: segment_x_end,
                });
            }
            current_text = ch.char.to_string();
            current_formatting = formatting;
            segment_y = ch.y;
            segment_x = ch.x;
            segment_x_end = ch.x + ch.width;
        } else {
            // Same formatting, same span - continue segment
            current_text.push(ch.char);
            segment_x_end = ch.x + ch.width;
        }
    }

    // Flush final segment
    if !current_text.is_empty() && current_formatting.is_some() {
        let text = current_text.trim().to_string();
        if !text.is_empty() {
            segments.push(TextSegment {
                text,
                is_deletion: current_formatting == Some("strikethrough"),
                page: current_page,
                y_pos: segment_y,
                x_pos: segment_x,
                x_end: segment_x_end,
            });
        }
    }

    segments
}

// =============================================================================
// Multi-line Merging
// =============================================================================

fn compute_page_metrics(
    colored_chars: &[ColoredChar],
    page_widths: &HashMap<usize, f32>,
    config: Config,
) -> HashMap<usize, PageMetrics> {
    let mut sums: HashMap<usize, (f32, usize, f32)> = HashMap::new();

    for ch in colored_chars {
        let entry = sums.entry(ch.page).or_insert((0.0, 0, 0.0));
        entry.0 += ch.height;
        entry.1 += 1;
        entry.2 = entry.2.max(ch.x + ch.width);
    }

    let mut metrics = HashMap::new();
    for (page, (height_sum, count, max_x)) in sums {
        let line_height = if count > 0 {
            height_sum / count as f32
        } else {
            config.line_height_fallback
        };
        let mut page_width = page_widths.get(&page).copied().unwrap_or(0.0);
        if max_x > page_width {
            page_width = max_x;
        }
        if page_width <= 0.0 {
            page_width = config.page_width_fallback;
        }
        metrics.insert(
            page,
            PageMetrics {
                line_height,
                page_width,
            },
        );
    }

    metrics
}

#[allow(dead_code)]
fn merge_multiline_segments(
    segments: Vec<TextSegment>,
    page_metrics: &HashMap<usize, PageMetrics>,
    config: Config,
) -> Vec<TextSegment> {
    if segments.is_empty() {
        return segments;
    }

    let mut sorted = segments;
    sorted.sort_by(|a, b| {
        let page_cmp = a.page.cmp(&b.page);
        if page_cmp != std::cmp::Ordering::Equal {
            return page_cmp;
        }
        let y_cmp = a
            .y_pos
            .partial_cmp(&b.y_pos)
            .unwrap_or(std::cmp::Ordering::Equal);
        if y_cmp != std::cmp::Ordering::Equal {
            return y_cmp;
        }
        a.x_pos
            .partial_cmp(&b.x_pos)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let mut merged = Vec::new();
    let mut i = 0;

    while i < sorted.len() {
        let current = &sorted[i];
        let metrics = page_metrics
            .get(&current.page)
            .copied()
            .unwrap_or(PageMetrics {
                line_height: config.line_height_fallback,
                page_width: config.page_width_fallback,
            });
        let mut combined_text = vec![current.text.clone()];
        let mut last_y = current.y_pos;
        let mut last_x_end = current.x_end;

        let mut j = i + 1;
        while j < sorted.len() {
            let next_seg = &sorted[j];

            if next_seg.page != current.page || next_seg.is_deletion != current.is_deletion {
                break;
            }

            let y_diff = next_seg.y_pos - last_y;

            if y_diff.abs() < config.same_line_y_tolerance {
                let x_gap = next_seg.x_pos - last_x_end;
                if x_gap > 0.0 && x_gap < config.merge_x_gap_max {
                    combined_text.push(next_seg.text.clone());
                    last_y = next_seg.y_pos;
                    last_x_end = next_seg.x_end;
                    j += 1;
                    continue;
                } else {
                    break;
                }
            }

            let is_next_line = y_diff > metrics.line_height * config.merge_line_height_min_ratio
                && y_diff < metrics.line_height * config.merge_line_height_max_ratio;
            let ends_at_margin = last_x_end > metrics.page_width * (1.0 - config.margin_end_ratio);
            let starts_at_margin = next_seg.x_pos < metrics.page_width * config.margin_start_ratio;

            if is_next_line && ends_at_margin && starts_at_margin {
                combined_text.push(next_seg.text.clone());
                last_y = next_seg.y_pos;
                last_x_end = next_seg.x_end;
                j += 1;
            } else {
                break;
            }
        }

        merged.push(TextSegment {
            text: combined_text.join(" "),
            is_deletion: current.is_deletion,
            page: current.page,
            y_pos: current.y_pos,
            x_pos: current.x_pos,
            x_end: last_x_end,
        });

        i = j;
    }

    merged
}

// =============================================================================
// Redline Grouping
// =============================================================================

fn group_segments_to_redlines(
    segments: Vec<TextSegment>,
    _page_metrics: &HashMap<usize, PageMetrics>,
    config: Config,
) -> Vec<NifRedline> {
    if segments.is_empty() {
        return Vec::new();
    }

    // Skip multi-line merging to match Python behavior (Python does not merge across lines)
    let mut sorted = segments;
    sorted.sort_by(|a, b| {
        let page_cmp = a.page.cmp(&b.page);
        if page_cmp != std::cmp::Ordering::Equal {
            return page_cmp;
        }
        let y_cmp = a
            .y_pos
            .partial_cmp(&b.y_pos)
            .unwrap_or(std::cmp::Ordering::Equal);
        if y_cmp != std::cmp::Ordering::Equal {
            return y_cmp;
        }
        a.x_pos
            .partial_cmp(&b.x_pos)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let mut redlines = Vec::new();
    let mut i = 0;

    while i < sorted.len() {
        let seg = &sorted[i];
        let mut paired = false;

        if i + 1 < sorted.len() {
            let next_seg = &sorted[i + 1];
            let y_diff = (seg.y_pos - next_seg.y_pos).abs();
            let same_line = y_diff < config.same_line_y_tolerance && seg.page == next_seg.page;
            let x_gap = next_seg.x_pos - seg.x_end;
            let x_adjacent = x_gap >= 0.0 && x_gap < config.pair_x_gap_max;

            if same_line && x_adjacent && seg.is_deletion && !next_seg.is_deletion {
                redlines.push(NifRedline {
                    r#type: "paired".to_string(),
                    deletion: Some(seg.text.clone()),
                    insertion: Some(next_seg.text.clone()),
                    location: format!("page {}", seg.page + 1),
                });
                i += 2;
                paired = true;
            }
        }

        if !paired {
            if seg.is_deletion {
                redlines.push(NifRedline {
                    r#type: "deletion".to_string(),
                    deletion: Some(seg.text.clone()),
                    insertion: None,
                    location: format!("page {}", seg.page + 1),
                });
            } else {
                redlines.push(NifRedline {
                    r#type: "insertion".to_string(),
                    deletion: None,
                    insertion: Some(seg.text.clone()),
                    location: format!("page {}", seg.page + 1),
                });
            }
            i += 1;
        }
    }

    redlines
}

// =============================================================================
// Main Extraction Logic
// =============================================================================

/// Check if a page has any redlines by examining if any colored character has formatting.
/// Returns true immediately upon finding the first redline.
fn page_has_redlines(
    formatting_bars: &[FormattingBar],
    colored_chars: &[ColoredChar],
    page: usize,
) -> bool {
    let page_bars: Vec<_> = formatting_bars.iter().filter(|b| b.page == page).collect();

    if page_bars.is_empty() {
        return false;
    }

    for ch in colored_chars.iter().filter(|c| c.page == page) {
        let formatting = get_char_formatting(
            ch.x,
            ch.width,
            ch.bbox_y0,
            ch.bbox_y1,
            &page_bars.iter().copied().cloned().collect::<Vec<_>>(),
            page,
        );

        if formatting.is_some() {
            return true;
        }
    }

    false
}

/// Check if a PDF has any redlines, with early exit on first detection.
fn has_redlines_impl(pdf_data: &[u8], config: Config) -> Result<bool, String> {
    let doc =
        Document::from_bytes(pdf_data, "").map_err(|e| format!("Failed to open PDF: {}", e))?;

    let state = Rc::new(RefCell::new(CollectorState {
        formatting_bars: Vec::new(),
        colored_chars: Vec::new(),
        current_page: 0,
        next_span_id: 0,
        config,
    }));

    let pages = doc
        .pages()
        .map_err(|e| format!("Failed to get pages: {}", e))?;
    for (page_num, page_result) in pages.enumerate() {
        let page = page_result.map_err(|e| format!("Failed to load page: {}", e))?;

        // Clear state for new page (we only need to check one page at a time)
        {
            let mut state_ref = state.borrow_mut();
            state_ref.formatting_bars.clear();
            state_ref.colored_chars.clear();
            state_ref.current_page = page_num;
        }

        let collector = RedlineCollector::new(Rc::clone(&state));
        let device = Device::from_native(collector)
            .map_err(|e| format!("Failed to create device: {}", e))?;

        page.run(&device, &Matrix::IDENTITY)
            .map_err(|e| format!("Failed to run page: {}", e))?;

        // Check if this page has any redlines
        let (formatting_bars, colored_chars) = {
            let state_ref = state.borrow();
            (
                state_ref.formatting_bars.clone(),
                state_ref.colored_chars.clone(),
            )
        };

        if page_has_redlines(&formatting_bars, &colored_chars, page_num) {
            return Ok(true);
        }
    }

    Ok(false)
}

fn extract_redlines_impl(pdf_data: &[u8], config: Config) -> Result<NifRedlineOutput, String> {
    let doc =
        Document::from_bytes(pdf_data, "").map_err(|e| format!("Failed to open PDF: {}", e))?;

    let state = Rc::new(RefCell::new(CollectorState {
        formatting_bars: Vec::new(),
        colored_chars: Vec::new(),
        current_page: 0,
        next_span_id: 0,
        config,
    }));

    let pages = doc
        .pages()
        .map_err(|e| format!("Failed to get pages: {}", e))?;
    let mut page_widths: HashMap<usize, f32> = HashMap::new();

    for (page_num, page_result) in pages.enumerate() {
        let page = page_result.map_err(|e| format!("Failed to load page: {}", e))?;
        if let Ok(bounds) = page.bounds() {
            let width = (bounds.x1 - bounds.x0).abs();
            if width > 0.0 {
                page_widths.insert(page_num, width);
            }
        }

        {
            let mut state_ref = state.borrow_mut();
            state_ref.current_page = page_num;
        }

        let collector = RedlineCollector::new(Rc::clone(&state));
        let device = Device::from_native(collector)
            .map_err(|e| format!("Failed to create device: {}", e))?;

        page.run(&device, &Matrix::IDENTITY)
            .map_err(|e| format!("Failed to run page: {}", e))?;
    }

    let (formatting_bars, colored_chars) = {
        let state_ref = state.borrow();
        (
            state_ref.formatting_bars.clone(),
            state_ref.colored_chars.clone(),
        )
    };

    let segments = extract_text_segments(&colored_chars, &formatting_bars, config);
    let page_metrics = compute_page_metrics(&colored_chars, &page_widths, config);
    let redlines = group_segments_to_redlines(segments, &page_metrics, config);

    Ok(NifRedlineOutput { redlines })
}

// =============================================================================
// NIF Functions
// =============================================================================

/// Extract redlines from PDF binary data.
#[rustler::nif(schedule = "DirtyCpu")]
fn nif_extract_redlines_from_binary<'a>(
    env: Env<'a>,
    pdf_binary: rustler::Binary,
    opts: Term<'a>,
) -> NifResult<Term<'a>> {
    let config = config_from_term(opts);
    match extract_redlines_impl(pdf_binary.as_slice(), config) {
        Ok(output) => Ok((ok(), output).encode(env)),
        Err(msg) => Ok((error(), msg).encode(env)),
    }
}

/// Extract redlines from a PDF file path.
#[rustler::nif(schedule = "DirtyCpu")]
fn nif_extract_redlines_from_path<'a>(
    env: Env<'a>,
    path: &str,
    opts: Term<'a>,
) -> NifResult<Term<'a>> {
    let pdf_data = std::fs::read(path)
        .map_err(|e| rustler::Error::Term(Box::new(format!("Failed to read file: {}", e))))?;

    let config = config_from_term(opts);
    match extract_redlines_impl(&pdf_data, config) {
        Ok(output) => Ok((ok(), output).encode(env)),
        Err(msg) => Ok((error(), msg).encode(env)),
    }
}

/// Check if a PDF file has any redlines (early exit on first detection).
#[rustler::nif(schedule = "DirtyCpu")]
fn nif_has_redlines_from_path<'a>(env: Env<'a>, path: &str, opts: Term<'a>) -> NifResult<Term<'a>> {
    let pdf_data = std::fs::read(path)
        .map_err(|e| rustler::Error::Term(Box::new(format!("Failed to read file: {}", e))))?;

    let config = config_from_term(opts);
    match has_redlines_impl(&pdf_data, config) {
        Ok(has_redlines) => Ok((ok(), has_redlines).encode(env)),
        Err(msg) => Ok((error(), msg).encode(env)),
    }
}

/// Check if PDF binary data has any redlines (early exit on first detection).
#[rustler::nif(schedule = "DirtyCpu")]
fn nif_has_redlines_from_binary<'a>(
    env: Env<'a>,
    pdf_binary: rustler::Binary,
    opts: Term<'a>,
) -> NifResult<Term<'a>> {
    let config = config_from_term(opts);
    match has_redlines_impl(pdf_binary.as_slice(), config) {
        Ok(has_redlines) => Ok((ok(), has_redlines).encode(env)),
        Err(msg) => Ok((error(), msg).encode(env)),
    }
}
