# rust-revise

Getting back into Rust after time away, coming from Zig. Not a tutorial repo — a small
project built to force real decisions, plus the notes that came out of building it.

## What's here

| Path | What it is |
|---|---|
| `src/` | A terminal file manager (see below) |
| `notes.md` | The reference that accumulated while building it — the useful artifact |
| `RUST_REVISION.md` | The original revision plan the work started from |
| `SCANNER_PLAN.md` | A staged plan for a follow-on scanner project |

## The project — terminal file manager

A single-keypress menu over four file operations. Deliberately small; the point was the
mechanics underneath, not the feature set.

```bash
cargo run
```

```
Choose an option:

1) Create a file.
2) Read a file.
3) Update a file.
4) Delete a file.

q) Exit.
```

- **Create** — write content to a new file
- **Read** — print contents, with a countdown before returning to the menu
- **Update** — open `$EDITOR` (falls back to `$VISUAL`, then `vi`)
- **Delete** — remove the file

### Layout

```
src/
├── main.rs      module wiring + the input loop
├── menu.rs      the Menu enum, Display, byte-to-variant parsing
├── files.rs     the four file operations
├── termios.rs   cbreak mode via libc
└── utils.rs     cls, prompt
```

### What it exercises

- `Read`/`Write` as **traits**, and why methods vanish without the import
- `Result`, `?`, and holding a `Result` so cleanup runs before propagating
- Enums with `Display`, exhaustive matching, `Option` at the parse boundary
- Moves — returning owned data without copying it
- FFI: `libc` termios, `unsafe` at the boundary, `MaybeUninit` for an output parameter
- **cbreak mode by hand** — clearing `ICANON` and `ECHO` only, rather than full raw mode,
  so `Ctrl-C` and `println!` keep working
- Spawning a child process that inherits the terminal (`Command::status`)
- Splitting a single file into modules — `mod` vs `use`, visibility, path prefixes

Unix only — termios doesn't exist on Windows.

Error handling is deliberately thin (`expect` in the file operations). It's a learning
project; the interesting parts were elsewhere.

## Notes

`notes.md` is the real output of this repo. It covers ownership and moves with measured
numbers, the `fs::` vs `File` decision, `libc` FFI, bitfield manipulation, cbreak vs raw
mode, the module system, and a running list of the traps actually hit along the way.
