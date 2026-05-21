# halide

A Rust CLI for managing analog film archives. Enforces consistent naming, directory structure, and metadata so every roll can be located — physically and digitally — without guessing.

## Concepts

**The roll is the atomic archival unit.** Everything else (scans, exports, notes) derives from it.

**UID** — a stable, immutable identifier derived from the development date:

```
2026-05-13_01
```

**Roll folder** — the UID plus human-readable metadata:

```
2026-05-13_01_HP5+@1600
```

**Frame filenames** — every scan inherits the roll UID:

```
2026-05-13_01_f01.tif
2026-05-13_01_f24.tif
```

## Archive structure

```
Film/
├── Rolls/
│   └── 2026/
│       ├── 2026-05-13_01_HP5+@1600/
│       │   ├── raw_scans/          ← scanner masters, never edited
│       │   ├── edits/
│       │   ├── exports/
│       │   ├── contact_sheet/
│       │   ├── metadata/
│       │   │   └── roll.toml       ← source of truth
│       │   └── notes.md
│       └── 2026-05-13_02_TriX@800/
├── Exports/
└── Projects/
```

## Installation

```bash
cargo install --path .
```

## Setup

Run once after installation:

```bash
halide init
```

Prompts for your archive root path and scanner defaults, then creates the top-level directory structure and writes `~/.config/halide/config.toml`.

## Workflow

### 1. Before scanning — create the roll

```bash
halide new
```

Prompts for development date, film stock, EI, camera, developer, and notes. Auto-detects the next roll number for the day. Creates the full directory scaffold and `metadata/roll.toml`.

### 2. After scanning — ingest the files

```bash
halide ingest /path/to/scanner/output
```

Or run without arguments to be prompted for the source directory:

```bash
halide ingest
```

Shows all rolls sorted by empty first, non-empty at the bottom. Moves TIFF files into `raw_scans/` and renames them to the canonical frame format. Writes scanner provenance into `roll.toml`. Handles cross-filesystem moves.

### 3. Add metadata and notes

```bash
halide note
```

Select a roll, then choose to open `roll.toml` in `$EDITOR` or step through frames interactively to add per-frame notes.

### 4. Verify integrity

```bash
halide verify
```

Checks all rolls for duplicate UIDs, missing metadata files, and frame sequence gaps.

## Metadata format

`metadata/roll.toml`:

```toml
uid = "2026-05-13_01"
film = "HP5+"
rated_iso = 1600
camera = "Leica M3"
lens = "50 Summicron"
developer = "HC-110B"
developed_date = "2026-05-13"
shot_date_range = "2026-03 → 2026-04"
notes = "Juniper first day of school. NYC trip."
tags = ["juniper", "nyc"]

[scan]
scanner = "Coolscan 5000"
scan_software = "VueScan"
dpi = 4000
bit_depth = 16
infrared_cleaning = true
multi_sampling = false

[[frames]]
id = "f01"
notes = "slightly underexposed"

[[frames]]
id = "f12"
notes = "strong — print this one"
```

## Config

`~/.config/halide/config.toml` — created by `halide init`, edit directly to add film stocks or change defaults.

## Rules

- `raw_scans/` is append-only — never edit files in place
- UIDs are immutable after creation
- Frame numbering is sequential from the sort order of the source files
- Physical sleeve UID matches digital UID exactly
