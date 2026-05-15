# Roadmap

## Phase 1 — Core CLI (current)

- [x] `halide init` — config and archive scaffold
- [x] `halide new` — roll directory and metadata creation
- [x] `halide ingest` — TIFF ingestion with canonical renaming
- [x] `halide note` — roll and frame metadata editing
- [x] `halide verify` — archive integrity checks

## Phase 2 — Search and Discovery

- [ ] `halide find` — search rolls by film stock, tags, date range, camera, notes
- [ ] `halide list` — tabular summary of all rolls with key fields
- [ ] Filesystem-first indexing; SQLite as optional acceleration layer

## Phase 3 — Export and Output

- [ ] `halide contact-sheet` — generate proof sheets from raw scans
- [ ] `halide export` — batch copy selects to `Exports/` with canonical names
- [ ] XMP/EXIF sidecar generation for Lightroom and Capture One compatibility

## Phase 4 — TUI

- [ ] Roll browser with frame preview navigation (`ratatui`)
- [ ] Inline frame note editing without leaving the interface
- [ ] Live verify status per roll

## Phase 5 — Archive Integrity

- [ ] Checksum manifests (`metadata/checksums.txt`) generated on ingest
- [ ] `halide verify --checksums` to detect bitrot or silent corruption
- [ ] Rescan tracking — record when a roll was rescanned and with what settings

## Phase 6 — Extended Metadata

- [ ] Camera body and lens registry in config for autocomplete
- [ ] Developer and dilution registry
- [ ] Print history tracking per frame
- [ ] `halide log` — append timestamped events to a roll (e.g. "printed f12 on Ilford MGRC 8x10")

## Deferred / Under Consideration

- AI-assisted tagging
- Duplicate scan detection
- Static gallery generation
- iPhone capture correlation by date range
- OCR from physical sleeve scans
