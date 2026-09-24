# nettle

Diagnostics for compilers and other tools that point into source text, for
[Meadow](https://github.com/mcdearman/meadow). A report has labelled spans
with arrows and messages, can span several files and many lines, and can end
with notes and help. It is drawn in colour or without.

This package is a port of Rust's
[`ariadne`](https://github.com/zesterer/ariadne) 0.6.0. It draws reports
character for character, and escape for escape, as the crate does. Text is
measured with [unicodeWidth](https://github.com/mcdearman/UnicodeWidth),
the port of the `unicode-width` version the crate uses.

## Install

```sh
meadow add mcdearman/Nettle
```

## Use

```meadow
use Nettle

def text = "fun add x y = x + y\n\ndef main = add 1 \"two\"\n"

def report =
  build Error (spanIn "main.mw" 38 43)
    |> withConfig (config |> withColor False)
    |> withNumericCode 3
    |> withMessage "mismatched types"
    |> withLabel (label (spanIn "main.mw" 4 7) |> labelMessage "`add` takes `Int`s")
    |> withLabel (label (spanIn "main.mw" 38 43) |> labelMessage "this is a `String`")
    |> withNote "convert it with `toInt`"

def main = print report (namedSource "main.mw" (source text))
```

```text
[03] Error: mismatched types
   ╭─[ main.mw:3:18 ]
   │
 1 │ fun add x y = x + y
   │     ─┬─  
   │      ╰─── `add` takes `Int`s
   │ 
 3 │ def main = add 1 "two"
   │                  ──┬──  
   │                    ╰──── this is a `String`
   │ 
   │ Note: convert it with `toInt`
───╯
```

The crate's builders change values in place. Here every function returns a
new value, taking it last so that calls chain with `|>`.

### Reports

- Start with `build kind span`. The kind is `Error`, `Warning`, `Advice` or
  `Custom name color`.
- Add to it with:
  - `withMessage`;
  - `withCode` (a string) or `withNumericCode` (zero-padded, as the crate
    pads it);
  - `withLabel` and `withLabels`;
  - `withNote`, `withNotes` and `setNote`;
  - `withHelp`, `withHelps` and `setHelp`;
  - `withConfig`.
- Draw it with `render report cache`, which gives the text, or print it with
  `print report cache`.
  `finish` is there for the crate's shape, and changes nothing.

### Spans, labels and sources

- **Spans:** `spanIn id start stop` covers `start` up to `stop` in the source
  `id`. A report about one unnamed source can use `span start stop`, whose id
  is `()`. Offsets count characters unless the configuration says bytes.
- **Labels:** `label span`. Change them with:
  - `labelMessage`;
  - `labelColor`;
  - `labelOrder`, which sorts labels, lowest first;
  - `labelPriority`, which decides whose colour shows where spans overlap.
- **Sources:** `source text`, split into lines at any Unicode line
  terminator. `withDisplayLineOffset n` numbers its lines from `n + 1` in a
  report's heading.
- **Caches** give a report its sources. A cache is a record of `fetch` and
  `display`, so any can be written by hand. The ready-made ones are:
  - `sourceCache`, for spans made with `span`;
  - `namedSource name source`;
  - `sources [(name, text)]`;
  - `fnCache get`;
  - `fileCache paths`, which reads the files when it is made.

### Configuration

Start from `config` and change it with:

- `withColor`;
- `withCompact`;
- `withCharSet` (`Unicode` or `Ascii`);
- `withIndexType` (`IndexType.Char` or `IndexType.Byte`);
- `withLabelAttach` (`Start`, `Middle` or `End`);
- `withCrossGap`;
- `withUnderlines`;
- `withMultilineArrows`;
- `withTabWidth`.

`withColor False` drops the report's own colours, but not a custom kind's.
It drops a label's colour only if the label is added after the configuration
is set: a label added before it is still drawn in colour. This is how the
crate behaves.

### Colours

A `Color` is one of:

- `Primary`, the terminal's default;
- the eight named colours, such as `Red`, and their `Bright` variants;
- `Fixed n`, a numbered colour from 0 to 255;
- `Rgb r g b`.

`fg color text` and `bg color text` paint any text. `colorGenerator` and
`nextColor` give a sequence of distinct colours for labels, as the crate's
`ColorGenerator` does.

### Differences from the crate

- A label whose span ends before it starts is taken as empty; the crate
  panics.
- A tab width of 0 is taken as 1; the crate panics.
- A byte offset inside a character counts that character as before it; the
  crate panics.
- A source a cache cannot give is left out silently. The crate also prints a
  warning to standard error, which Meadow's console does not have. For the
  same reason, `print` writes to standard output only.
- `FnCache` does not cache, since `get` is pure.

## How it's made

The modules in `src/` translate the crate's `write.rs` step by step. This
includes its known quirks: labels that go back up a file start a new section,
and a report with no labels shows no notes. The escape sequences are yansi's.

**`src/Cases.mw`** holds three kinds of checks:

- **1,200 random reports.** Each records the steps that build it and what the
  crate draws for it. The reports cover:
  - every configuration, and builder steps in random orders;
  - several sources, and sources the cache lacks;
  - multi-line, overlapping, empty and out-of-range spans;
  - label orders and priorities;
  - tabs, and wide, combining and zero-width text;
  - every line terminator.
- **200 colour generators.**
- **300 texts split into lines**, with offset lookups.

Run `scripts/generate.sh` to regenerate; it needs a Rust toolchain.

## Licence

MIT, like ariadne: see [LICENSE](LICENSE), [LICENSE-yansi](LICENSE-yansi) and
[COPYRIGHT](COPYRIGHT).
