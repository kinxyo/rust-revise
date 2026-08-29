# rust-revise

Relearning Rust properly, coming from Zig. Not a tutorial repo — a working log of projects
built to force specific concepts, plus the notes that came out of them.

## What's here

| Path | What it is |
|---|---|
| `src/main.rs` | **Project 1** — a terminal file manager (see below) |
| `notes.md` | The reference that accumulated while building it. The useful artifact. |
| `ROADMAP.md` | The four-part plan and what each part teaches |
| `SCANNER_PLAN.md` | Detail on project 2, the recursive file scanner |
| `RUST_REVISION.md` | The original one-day revision plan |

## Project 1 — terminal file manager

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
- **Update** — opens `$EDITOR` (falls back to `$VISUAL`, then `vi`)
- **Delete** — remove the file

### What it exercises

- `Read`/`Write` as **traits**, and why methods vanish without the import
- `Result`, `?`, and holding a `Result` so cleanup runs before propagating
- Enums with `Display`, exhaustive matching, `Option` at the parse boundary
- Moves — returning owned data without copying it
- FFI: `libc` termios, `unsafe` at the boundary, `MaybeUninit` for an output parameter
- **cbreak mode by hand** — clearing `ICANON` and `ECHO` only, rather than full raw mode,
  so `Ctrl-C` and `println!` keep working
- Spawning a child process that inherits the terminal (`Command::status`)

Unix only — termios doesn't exist on Windows.

Error handling is deliberately thin (`expect` in the file operations). It's a learning
project; the interesting parts were elsewhere.

## Notes

`notes.md` is the real output of this repo. It covers ownership and moves with measured
numbers, the `fs::` vs `File` decision, `libc` FFI, bitfield manipulation, cbreak vs raw
mode, and a running list of the traps actually hit along the way.

## What's next

See `ROADMAP.md`. Short version: a scanner engine (threads, `Send`/`Sync`, channels), then
an HTTP service wrapping it (async, `spawn_blocking`, shutdown), then real image work on
top.
