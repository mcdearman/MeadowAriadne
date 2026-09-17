//! Writes `src/Cases.mw` for ariadne.
//!
//! ```text
//! cargo run --release -- <package root>
//! ```
//!
//! Random reports — sources, labels, notes, help, codes, kinds and
//! configurations, built in random orders — with what the crate draws for
//! them; the colours of random colour generators; and how random texts split
//! into lines. The library is ported by hand into `src/`, and the crate's
//! source is fingerprinted, along with the part of yansi that paints for it.

use ariadne::{
    CharSet, Color, ColorGenerator, Config, IndexType, Label, LabelAttach, Report, ReportKind,
    Source,
};
use std::fmt::Write as _;
use std::ops::Range;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::PathBuf;

/// The crate version pinned in `Cargo.toml`.
const UPSTREAM_VERSION: &str = "0.6.0";

/// The fingerprint of the sources `src/` ports.
const SOURCES: u64 = 0x295a_02cc_f157_fe11;

fn main() {
    let root = PathBuf::from(std::env::args().nth(1).unwrap_or_else(|| "../..".into()));

    let print = fingerprint(include_str!(concat!(env!("OUT_DIR"), "/sources.rs.txt")));
    if print != SOURCES {
        eprintln!(
            "error: ariadne is not the version src/ ports.\n\
             Compare its source in {} with the previous version, carry any change\n\
             into src/, then set SOURCES in scripts/generate/src/main.rs to\n\
             {print:#x}",
            env!("UPSTREAM_DIR")
        );
        std::process::exit(1);
    }

    // Some random reports are ones the crate panics on (a byte offset inside
    // a character); they are dropped, quietly.
    std::panic::set_hook(Box::new(|_| {}));
    yansi::enable();

    let cases = cases();
    let path = root.join("src/Cases.mw");
    std::fs::write(&path, &cases).unwrap();
    eprintln!("wrote {} ({} bytes)", path.display(), cases.len());
}

/// FNV-1a: stable across builds, which `DefaultHasher` does not promise.
fn fingerprint(text: &str) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in text.bytes() {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    h
}

// --- encoding -----------------------------------------------------------------------

/// A number as `digits` base-64 digits, most significant first, each digit the
/// character `'0' + d`: `'0'` to `'o'`, one contiguous run of ASCII.
fn digits(out: &mut String, value: u64, digits: u32) {
    assert!(
        value < 1 << (6 * digits),
        "{value} does not fit in {digits} digits"
    );
    for k in (0..digits).rev() {
        out.push(char::from(b'0' + ((value >> (6 * k)) & 63) as u8));
    }
}

/// A signed number, offset to fit `digits` digits.
fn signed(out: &mut String, value: i64, n: u32) {
    let half = 1i64 << (6 * n - 1);
    assert!((-half..half).contains(&value), "{value} does not fit");
    digits(out, (value + half) as u64, n);
}

/// A string, as its length in bytes (3 digits) and then its bytes.
fn text(out: &mut String, s: &str) {
    digits(out, s.len() as u64, 3);
    out.push_str(s);
}

fn flag(out: &mut String, b: bool) {
    digits(out, u64::from(b), 1);
}

/// `text` as one Meadow string literal, broken with `\`-newline every `width`
/// characters. Printable ASCII, box-drawing characters and CJK are written
/// raw, and everything else escaped; a space that would start a line is
/// `\x20`, since a continuation drops leading whitespace.
fn long_literal(text: &str, width: usize) -> String {
    let mut out = String::with_capacity(text.len() + text.len() / width * 4 + 2);
    out.push('"');
    for (i, c) in text.chars().enumerate() {
        let line_start = i > 0 && i % width == 0;
        if line_start {
            out.push_str("\\\n    ");
        }
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '$' => out.push_str("\\$"),
            ' ' if line_start => out.push_str("\\x20"),
            ' '..='~' | '\u{2500}'..='\u{259F}' => out.push(c),
            '\u{3040}'..='\u{30FF}' | '\u{4E00}'..='\u{9FFF}' => out.push(c),
            _ => {
                let _ = write!(out, "\\u{{{:X}}}", u32::from(c));
            }
        }
    }
    out.push('"');
    out
}

// --- inputs -------------------------------------------------------------------------

/// A small deterministic generator, so that the cases are the same on every run.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        // xorshift64*
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_f491_4f6c_dd1d)
    }

    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }

    fn chance(&mut self, percent: u64) -> bool {
        self.below(100) < percent
    }

    fn pick<T: Copy>(&mut self, xs: &[T]) -> T {
        xs[self.below(xs.len() as u64) as usize]
    }
}

const LINES: &[&str] = &[
    "fn main() {",
    "    let x = 5;",
    "\tlet y = x + 1;",
    "\t\tif a\t{ b }",
    "}",
    "",
    "   ",
    "let s = \"日本語テキスト\";",
    "emoji 😀 and 👨‍👩‍👧 here",
    "e\u{301}le\u{300}ve",
    "trailing spaces   ",
    "a\u{7}bell",
    "zero\u{200B}width",
    "wide\u{3000}space",
    "nbsp\u{A0}x",
    "x",
    "match value { Some(v) => v, None => 0 }",
    "│ box ─ chars ┼",
];

const SEPARATORS: &[&str] = &[
    "\n", "\n", "\n", "\n", "\n", "\n", "\r\n", "\r", "\u{B}", "\u{C}", "\u{85}", "\u{2028}",
    "\u{2029}",
];

/// Short lines of wide characters and tabs, where labels crowd together.
const DENSE_LINES: &[&str] = &[
    "日本語\t日本語",
    "\t\tx = y",
    "テキスト テキスト",
    "ab日c",
    "x",
    "fn f(a: 日) {",
];

fn random_source(rng: &mut Rng, dense: bool) -> String {
    let lines = if dense {
        1 + rng.below(4)
    } else {
        rng.below(12)
    };
    let pool = if dense { DENSE_LINES } else { LINES };
    let mut s = String::new();
    for _ in 0..lines {
        s.push_str(rng.pick(pool));
        s.push_str(rng.pick(SEPARATORS));
    }
    if rng.chance(40) {
        s.push_str(rng.pick(pool));
    }
    s
}

const MESSAGES: &[&str] = &[
    "expected `i32`",
    "this is found to be of type `String`",
    "",
    "日本語のメッセージ",
    "first line\nsecond line",
    "carriage\r\nreturn",
    "ends with newline\n",
    "\n",
    "a\n\nb",
    "tab\there",
    "emoji 😀",
    "x",
];

const CODES: &[&str] = &["E0308", "", "é", "x", "3", "W12", "日"];

const NAMES: &[&str] = &["Bug", "", "Lint", "注意"];

const NAMED: [Color; 18] = [
    Color::Primary,
    Color::Black,
    Color::Red,
    Color::Green,
    Color::Yellow,
    Color::Blue,
    Color::Magenta,
    Color::Cyan,
    Color::White,
    Color::BrightBlack,
    Color::BrightRed,
    Color::BrightGreen,
    Color::BrightYellow,
    Color::BrightBlue,
    Color::BrightMagenta,
    Color::BrightCyan,
    Color::BrightWhite,
    Color::Primary,
];

fn random_color(rng: &mut Rng, out: &mut String) -> Color {
    match rng.below(4) {
        0 | 1 => {
            let k = rng.below(17);
            digits(out, k, 1);
            NAMED[k as usize]
        }
        2 => {
            let n = rng.below(256) as u8;
            digits(out, 17, 1);
            digits(out, n.into(), 2);
            Color::Fixed(n)
        }
        _ => {
            let (r, g, b) = (
                rng.below(256) as u8,
                rng.below(256) as u8,
                rng.below(256) as u8,
            );
            digits(out, 18, 1);
            for v in [r, g, b] {
                digits(out, v.into(), 2);
            }
            Color::Rgb(r, g, b)
        }
    }
}

fn random_config(rng: &mut Rng, out: &mut String) -> Config {
    let cross_gap = rng.chance(50);
    let attach = rng.below(3);
    let compact = rng.chance(30);
    let underlines = rng.chance(70);
    let multiline_arrows = rng.chance(70);
    let color = rng.chance(60);
    let tab_width = 1 + rng.below(8);
    let ascii = rng.chance(30);
    let bytes = rng.chance(30);
    for b in [
        cross_gap,
        compact,
        underlines,
        multiline_arrows,
        color,
        ascii,
        bytes,
    ] {
        flag(out, b);
    }
    digits(out, attach, 1);
    digits(out, tab_width, 1);
    Config::default()
        .with_cross_gap(cross_gap)
        .with_label_attach(
            [LabelAttach::Start, LabelAttach::Middle, LabelAttach::End][attach as usize],
        )
        .with_compact(compact)
        .with_underlines(underlines)
        .with_multiline_arrows(multiline_arrows)
        .with_color(color)
        .with_tab_width(tab_width as usize)
        .with_char_set(if ascii {
            CharSet::Ascii
        } else {
            CharSet::Unicode
        })
        .with_index_type(if bytes {
            IndexType::Byte
        } else {
            IndexType::Char
        })
}

/// An offset into a source of `len` units: mostly inside it, now and then
/// just past its end.
fn random_offset(rng: &mut Rng, len: usize) -> usize {
    (rng.below(len as u64 + 3)) as usize
}

/// A span of the source `text`, as characters or (mostly on character
/// boundaries) bytes.
///
/// With `anchors`, the span starts at one of them and has one of a few
/// lengths, so that spans share starts, ends and lengths.
fn random_range(rng: &mut Rng, text: &str, anchors: Option<&[usize]>) -> Range<usize> {
    let chars = text.chars().count();
    let bytes = text.len();
    let byte_mode = rng.chance(50);
    let len = if byte_mode { bytes } else { chars };
    if let Some(anchors) = anchors {
        let start = rng.pick(anchors);
        return start..start + rng.pick(&[0, 1, 2, 3, 6, 14]);
    }
    let start = random_offset(rng, len);
    let end = match rng.below(4) {
        0 => start,
        1 => start + rng.below(4) as usize,
        2 => start + rng.below(40) as usize,
        _ => start.max(random_offset(rng, len)),
    };
    if byte_mode && rng.chance(90) {
        let snap = |x: usize| {
            let mut x = x.min(bytes + 2);
            while x <= bytes && !text.is_char_boundary(x) {
                x += 1;
            }
            x
        };
        snap(start)..snap(end).max(snap(start))
    } else {
        start..end
    }
}

/// One report, with ids (`String`) or without (`()`), written as the steps
/// that build it. Returns the drawing, or `None` if the crate panicked.
fn random_report(rng: &mut Rng, out: &mut String) -> Option<String> {
    let with_ids = rng.chance(75);
    let dense = rng.chance(40);
    let anchors: Vec<usize> = (0..3).map(|_| rng.below(16) as usize).collect();
    let anchors = dense.then_some(anchors.as_slice());
    flag(out, with_ids);
    let count = if with_ids { 1 + rng.below(3) } else { 1 };
    digits(out, count, 1);
    let mut sources: Vec<(String, String, usize)> = Vec::new();
    for k in 0..count {
        let name = if k == 0 && rng.chance(20) {
            "日本/src.rs".to_string()
        } else {
            format!("file{k}.rs")
        };
        let body = random_source(rng, dense);
        let offset = if rng.chance(20) {
            rng.below(2000) as usize
        } else {
            0
        };
        text(out, &name);
        text(out, &body);
        digits(out, offset as u64, 2);
        sources.push((name, body, offset));
    }
    // Index `count` names a source the cache does not have.
    let pick_source = |rng: &mut Rng| -> usize {
        if with_ids && rng.chance(5) {
            count as usize
        } else {
            rng.below(count) as usize
        }
    };
    let text_of = |i: usize| -> &str { sources.get(i).map_or("", |s| s.1.as_str()) };
    let name_of =
        |i: usize| -> String { sources.get(i).map_or("missing.rs".into(), |s| s.0.clone()) };

    let kind_tag = rng.below(4);
    digits(out, kind_tag, 1);
    let kind = match kind_tag {
        0 => ReportKind::Error,
        1 => ReportKind::Warning,
        2 => ReportKind::Advice,
        _ => {
            let name = rng.pick(NAMES);
            text(out, name);
            ReportKind::Custom(name, random_color(rng, out))
        }
    };
    let span_source = pick_source(rng);
    let span = random_range(rng, text_of(span_source), None);
    digits(out, span_source as u64, 1);
    digits(out, span.start as u64, 2);
    digits(out, span.end as u64, 2);

    enum Builder<'a> {
        Ids(ariadne::ReportBuilder<'a, (String, Range<usize>)>),
        Plain(ariadne::ReportBuilder<'a, Range<usize>>),
    }
    macro_rules! apply {
        ($b:expr, |$x:ident| $body:expr) => {
            match $b {
                Builder::Ids($x) => Builder::Ids($body),
                Builder::Plain($x) => Builder::Plain($body),
            }
        };
    }
    macro_rules! apply_mut {
        ($b:expr, |$x:ident| $body:expr) => {
            match &mut $b {
                Builder::Ids($x) => $body,
                Builder::Plain($x) => $body,
            }
        };
    }
    let mut builder = if with_ids {
        Builder::Ids(Report::build(kind, (name_of(span_source), span.clone())))
    } else {
        Builder::Plain(Report::build(kind, span.clone()))
    };

    let steps = if dense {
        4 + rng.below(10)
    } else {
        rng.below(12)
    };
    digits(out, steps, 1);
    for _ in 0..steps {
        let tag = if dense && rng.chance(60) {
            10 + rng.below(2)
        } else {
            rng.below(12)
        };
        digits(out, tag, 1);
        match tag {
            0 => {
                let code = rng.pick(CODES);
                text(out, code);
                builder = apply!(builder, |b| b.with_code(code));
            }
            1 => {
                let n = rng.below(300) as i64 - 150;
                signed(out, n, 2);
                builder = apply!(builder, |b| b.with_code(n));
            }
            2 => {
                let m = rng.pick(MESSAGES);
                text(out, m);
                builder = apply!(builder, |b| b.with_message(m));
            }
            3 => {
                let m = rng.pick(MESSAGES);
                text(out, m);
                builder = apply!(builder, |b| b.with_note(m));
            }
            4 => {
                let m = rng.pick(MESSAGES);
                text(out, m);
                builder = apply!(builder, |b| b.with_help(m));
            }
            5 => {
                let m = rng.pick(MESSAGES);
                text(out, m);
                apply_mut!(builder, |b| b.set_note(m));
            }
            6 => {
                let m = rng.pick(MESSAGES);
                text(out, m);
                apply_mut!(builder, |b| b.set_help(m));
            }
            7 => {
                let config = random_config(rng, out);
                builder = apply!(builder, |b| b.with_config(config));
            }
            8 => {
                let ms: Vec<&str> = (0..rng.below(3)).map(|_| rng.pick(MESSAGES)).collect();
                digits(out, ms.len() as u64, 1);
                for m in &ms {
                    text(out, m);
                }
                apply_mut!(builder, |b| b.with_notes(ms.iter()));
            }
            9 => {
                let ms: Vec<&str> = (0..rng.below(3)).map(|_| rng.pick(MESSAGES)).collect();
                digits(out, ms.len() as u64, 1);
                for m in &ms {
                    text(out, m);
                }
                apply_mut!(builder, |b| b.with_helps(ms.iter()));
            }
            _ => {
                // One label, or (tag 11) several at once.
                let n = if tag == 10 {
                    1
                } else {
                    rng.below(if dense { 6 } else { 4 })
                };
                if tag == 11 {
                    digits(out, n, 1);
                }
                let mut plain = Vec::new();
                let mut ids = Vec::new();
                for _ in 0..n {
                    let source = pick_source(rng);
                    let range = random_range(rng, text_of(source), anchors);
                    digits(out, source as u64, 1);
                    digits(out, range.start as u64, 2);
                    digits(out, range.end as u64, 2);
                    let msg = rng.chance(80).then(|| rng.pick(MESSAGES));
                    flag(out, msg.is_some());
                    if let Some(m) = msg {
                        text(out, m);
                    }
                    let has_color = rng.chance(70);
                    flag(out, has_color);
                    let color = has_color.then(|| random_color(rng, out));
                    let order = if rng.chance(if dense { 50 } else { 30 }) {
                        rng.below(7) as i64 - 3
                    } else {
                        0
                    };
                    let priority = if rng.chance(30) {
                        rng.below(7) as i64 - 3
                    } else {
                        0
                    };
                    signed(out, order, 1);
                    signed(out, priority, 1);
                    macro_rules! dress {
                        ($l:expr) => {{
                            let mut l = $l;
                            if let Some(m) = msg {
                                l = l.with_message(m);
                            }
                            if let Some(c) = color {
                                l = l.with_color(c);
                            }
                            if order != 0 {
                                l = l.with_order(order as i32);
                            }
                            if priority != 0 {
                                l = l.with_priority(priority as i32);
                            }
                            l
                        }};
                    }
                    plain.push(dress!(Label::new(range.clone())));
                    ids.push(dress!(Label::new((name_of(source), range))));
                }
                if tag == 10 {
                    builder = match builder {
                        Builder::Ids(b) => Builder::Ids(b.with_label(ids.pop().unwrap())),
                        Builder::Plain(b) => Builder::Plain(b.with_label(plain.pop().unwrap())),
                    };
                } else {
                    builder = match builder {
                        Builder::Ids(b) => Builder::Ids(b.with_labels(ids)),
                        Builder::Plain(b) => Builder::Plain(b.with_labels(plain)),
                    };
                }
            }
        }
    }

    let mut drawn = Vec::new();
    let result = catch_unwind(AssertUnwindSafe(|| match builder {
        Builder::Ids(b) => {
            let mut cache = ariadne::FnCache::new(
                (|id: &String| Err(format!("Failed to fetch source '{id}'"))) as fn(&_) -> _,
            )
            .with_sources(
                sources
                    .iter()
                    .map(|(name, body, offset)| {
                        (
                            name.clone(),
                            Source::from(body.clone()).with_display_line_offset(*offset),
                        )
                    })
                    .collect(),
            );
            b.finish().write(&mut cache, &mut drawn)
        }
        Builder::Plain(b) => {
            let (_, body, offset) = &sources[0];
            let source = Source::from(body.clone()).with_display_line_offset(*offset);
            b.finish().write(&source, &mut drawn)
        }
    }));
    match result {
        Ok(r) => {
            r.unwrap();
            Some(String::from_utf8(drawn).unwrap())
        }
        Err(_) => None,
    }
}

// --- colour generators --------------------------------------------------------------

fn color_cases(rng: &mut Rng, out: &mut String) -> usize {
    let count = 200;
    for _ in 0..count {
        let state = [
            rng.below(65536) as u16,
            rng.below(65536) as u16,
            rng.below(65536) as u16,
        ];
        for s in state {
            digits(out, s.into(), 3);
        }
        // Mostly in range; now and then NaN, negative, or above 1.
        let kind = match rng.below(5) {
            1 => 1,
            2 => 2,
            _ => 0,
        };
        let m = rng.below(80000);
        digits(out, kind, 1);
        digits(out, m, 3);
        let brightness = match kind {
            0 => m as f32 / 65536.0,
            1 => f32::NAN,
            _ => -(m as f32) / 65536.0,
        };
        let mut generator = ColorGenerator::from_state(state, brightness);
        let n = 1 + rng.below(20);
        digits(out, n, 1);
        for _ in 0..n {
            match generator.next() {
                Color::Fixed(v) => digits(out, v.into(), 2),
                other => panic!("a generator made {other:?}"),
            }
        }
    }
    count
}

// --- lines --------------------------------------------------------------------------

fn line_cases(rng: &mut Rng, out: &mut String) -> usize {
    let count = 300;
    for _ in 0..count {
        let body = random_source(rng, false);
        text(out, &body);
        let source = Source::from(body.as_str());
        digits(out, source.lines().len() as u64, 2);
        for line in source.lines() {
            digits(out, line.offset() as u64, 2);
            digits(out, line.len() as u64, 2);
            text(out, source.get_line_text(line).unwrap());
        }
        digits(out, source.len() as u64, 2);
        let queries = 6;
        for q in 0..queries {
            let bytes = q % 2 == 1;
            let len = if bytes { body.len() } else { source.len() };
            let offset = random_offset(rng, len);
            digits(out, offset as u64, 2);
            let found = if bytes {
                source.get_byte_line(offset)
            } else {
                source.get_offset_line(offset)
            };
            flag(out, found.is_some());
            if let Some((line, idx, col)) = found {
                digits(out, line.offset() as u64, 2);
                digits(out, idx as u64, 2);
                digits(out, col as u64, 2);
            }
        }
        let range = random_range(rng, &body, None);
        let lines = source.get_line_range(&range);
        for v in [range.start, range.end, lines.start, lines.end] {
            digits(out, v as u64, 2);
        }
    }
    count
}

// --- cases --------------------------------------------------------------------------

fn cases() -> String {
    let mut rng = Rng(0xa21a_d0e5_5eed_0042);
    let mut reports = String::new();
    let count = 1200;
    let mut made = 0;
    let mut dropped = 0;
    while made < count {
        let mut steps = String::new();
        match random_report(&mut rng, &mut steps) {
            Some(drawn) => {
                reports.push_str(&steps);
                text(&mut reports, &drawn);
                made += 1;
            }
            None => dropped += 1,
        }
    }
    eprintln!("{made} reports ({dropped} dropped: the crate panicked on them)");
    let mut colors = String::new();
    let color_count = color_cases(&mut rng, &mut colors);
    let mut lines = String::new();
    let line_count = line_cases(&mut rng, &mut lines);

    let mut out = String::new();
    let _ = writeln!(
        out,
        "-- GENERATED by scripts/generate.sh from ariadne {UPSTREAM_VERSION}.
-- Do not edit: run the script again instead.
--
-- Inputs, with what the crate makes of them, for `Tests.mw`: {count} random
-- reports, {color_count} colour generators and {line_count} texts split into lines.
--
-- Copyright Joshua Barretto and the ariadne contributors, and the Meadow
-- port's authors. Licensed under MIT: see LICENSE and COPYRIGHT.
--
-- A number is base-64 digits; a string is its length in bytes (3 digits) and
-- then its bytes.

-- Each report is written as its sources, its kind and span, and the steps
-- that build it, in the order `Tests.mw` reads and takes them, and then what
-- the crate draws.
@cfg(test)
@pub(pkg) def reports =
  {}

-- Each generator is its state and brightness, and the colours it makes.
@cfg(test)
@pub(pkg) def colors =
  {}

-- Each text is written with its lines, its length, lines found for offsets,
-- and the lines a span covers.
@cfg(test)
@pub(pkg) def lines =
  {}",
        long_literal(&reports, 96),
        long_literal(&colors, 96),
        long_literal(&lines, 96)
    );
    out
}
