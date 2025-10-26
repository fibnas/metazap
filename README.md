# Metazap

Metazap is a small CLI tool for recursively stripping metadata from PNG and JPEG images. It optionally creates backups, performs dry runs, and can run [oxipng](https://github.com/shssoichiro/oxipng) for lossless PNG recompression after the metadata zap.

## Features
- Scan a directory tree (recursion can be disabled with `--no-recursive`).
- Write back in-place or into a separate output directory.
- Optional `.bak` backups before overwriting files.
- Dry-run mode that prints the planned work without touching disk.
- Optional `oxipng` pass (`-z/--optimize`) for smaller PNGs.

## Build & Install
```bash
cargo build --release
# or install globally during development
cargo install --path .
```

## Usage
```bash
metazap [OPTIONS] [--input <DIR>] [--output <DIR>]
```

| Flag | Description |
| --- | --- |
| `-i, --input <DIR>` | Directory to scan (defaults to `.`). |
| `-o, --output <DIR>` | Optional destination directory; omitted means in-place overwrite. |
| `-r/--recursive`, `--no-recursive` | Toggle recursion (defaults to `true`). |
| `-d, --dry-run` | Log the planned work without touching files. |
| `-z, --optimize` | Run oxipng after zapping PNGs for extra savings. |
| `-b, --backup` | Copy in-place targets to `*.bak.<ext>` before writing. |

Examples:
```bash
# Rewrite the current directory in-place, keeping backups
metazap --backup

# Clean ~/Pictures into ./cleaned without touching the originals
metazap -i ~/Pictures -o ./cleaned

# Inspect exactly what would run before committing
metazap --dry-run --no-recursive
```

## Development
- `cargo fmt` keeps the style consistent.
- `cargo clippy -- -D warnings` is recommended for catching mistakes early.
- The current binary does not ship automated tests; consider adding integration tests that feed fixture images through the binary to protect against future regressions.
