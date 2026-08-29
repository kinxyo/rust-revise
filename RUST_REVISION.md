# Rust in One Day — Revision Plan

A one-day path from "I've forgotten most of it" to "I can start building." Each block is
**read briefly → drill in code → move on**. Do not read passively; the recall comes from typing.

Total: ~9 hours with breaks. Blocks 1–4 are the language. Block 5 is the project.

---

## Hour 0 — Setup (15 min)

If Rust isn't installed yet:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
rustc --version && cargo --version

rustup component add clippy rustfmt rust-analyzer
cargo install rustlings          # used in Block 1
```

Sanity check the existing crate (it's `edition = "2024"`, so you need a recent toolchain):

```bash
cargo run
```

Commands you'll live in all day:

| Command | Use |
|---|---|
| `cargo check` | Fast type-check, no codegen. Your inner loop. |
| `cargo clippy` | Lint. **Run constantly** — it re-teaches idioms faster than any book. |
| `cargo test` | Run tests. |
| `cargo fmt` | Format. |
| `cargo add <crate>` | Add a dependency. |
| `cargo doc --open` | Local docs for your deps. |

---

## Block 1 — Ownership, Borrowing, Lifetimes (2h)

This is the only part of Rust that is genuinely *different*. Everything else is a normal
modern language. Spend the time here.

### The three rules

1. Every value has exactly one **owner**. When the owner goes out of scope, the value is dropped (freed).
2. Assigning or passing a value **moves** it — unless the type is `Copy` (integers, bools, chars, `&T`, and tuples/arrays of those).
3. You may have **either** any number of `&T` (shared borrows) **or** exactly one `&mut T` (exclusive borrow) — never both.

```rust
let s = String::from("photo.cr2");
let t = s;                  // s MOVED into t; s is now unusable
// println!("{s}");         // compile error

let a = String::from("x");
let b = a.clone();          // explicit deep copy; both usable

let n = 5;
let m = n;                  // i32 is Copy; n still usable
```

### Borrowing

```rust
fn len(s: &String) -> usize { s.len() }        // borrows, doesn't take
fn push(s: &mut String) { s.push('!'); }       // exclusive borrow

let mut s = String::from("hi");
let r1 = &s;
let r2 = &s;          // fine — many shared borrows
println!("{r1} {r2}");
let r3 = &mut s;      // fine HERE: r1/r2 last used above (non-lexical lifetimes)
r3.push('!');
```

The borrow checker tracks *last use*, not scope end. Most "cannot borrow as mutable" errors
are fixed by shortening the shared borrow's lifespan or by restructuring so you don't hold a
reference across a mutation.

### Owned vs borrowed — memorize this table

| Owned | Borrowed | Notes |
|---|---|---|
| `String` | `&str` | Take `&str` in fn args, return `String`. |
| `Vec<T>` | `&[T]` | Take `&[T]` in fn args. |
| `PathBuf` | `&Path` | Same rule. **You will use these constantly.** |
| `T` | `&T` / `&mut T` | |

Rule of thumb: **accept the borrowed form, return the owned form.**

### Lifetimes

Usually elided. You write them in exactly two places:

```rust
// 1. A struct holding a reference
struct Scan<'a> {
    root: &'a Path,
}

// 2. A fn whose return borrows from an ambiguous input
fn longest<'a>(a: &'a str, b: &'a str) -> &'a str {
    if a.len() > b.len() { a } else { b }
}
```

`'a` means "lives at least as long as." `'static` means "lives for the whole program."
If lifetimes get painful, that's usually a signal to **own the data instead** (`String`
rather than `&str`) — perfectly idiomatic, especially in application code.

### Drill (60 min)

```bash
rustlings init && cd rustlings && rustlings
```

Do the sections: `variables`, `move_semantics`, `primitive_types`, `structs`, `enums`,
`strings`, `lifetimes`. Skip anything that's instantly obvious.

---

## Block 2 — Types, Traits, Errors (1.5h)

### Structs and enums

```rust
#[derive(Debug, Clone, PartialEq)]
struct ImageFile {
    path: PathBuf,
    bytes: u64,
    format: Format,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Format { Jpeg, Png, Cr2, Nef, Arw }

impl ImageFile {
    fn new(path: PathBuf, bytes: u64, format: Format) -> Self {
        Self { path, bytes, format }
    }
    fn is_raw(&self) -> bool {
        matches!(self.format, Format::Cr2 | Format::Nef | Format::Arw)
    }
}
```

Enums carry data — this is Rust's single best feature and the backbone of your job/state modelling:

```rust
enum JobState {
    Queued,
    Running { started: SystemTime, pct: u8 },
    Done(ImageFile),
    Failed { path: PathBuf, err: String },
}
```

### Pattern matching

```rust
match state {
    JobState::Queued => {}
    JobState::Running { pct, .. } if pct > 90 => println!("almost"),
    JobState::Running { .. } => {}
    JobState::Done(f) => println!("{}", f.path.display()),
    JobState::Failed { path, err } => eprintln!("{}: {err}", path.display()),
}

if let JobState::Done(f) = &state { /* ... */ }

let JobState::Done(f) = state else {   // let-else: bind or bail
    return Err(Error::NotFinished);
};
```

### Traits and generics

```rust
trait Validator {
    fn validate(&self, bytes: &[u8]) -> Result<Format, ValidationError>;
    fn name(&self) -> &str { "unnamed" }      // default method
}

// static dispatch — monomorphized, zero cost
fn run<V: Validator>(v: &V, b: &[u8]) { /* ... */ }
fn run2(v: &impl Validator, b: &[u8]) { /* ... */ }

// dynamic dispatch — vtable, needed for heterogeneous collections
fn run3(v: &dyn Validator) { /* ... */ }
let validators: Vec<Box<dyn Validator>> = vec![Box::new(JpegV), Box::new(PngV)];

// where-clause form for many bounds
fn process<T>(items: Vec<T>) where T: Validator + Send + 'static { }
```

Derives you'll actually use: `Debug, Clone, Copy, PartialEq, Eq, Hash, Default,
PartialOrd, Ord` — plus `Serialize, Deserialize` from serde.

### Error handling — the part that matters for this project

```rust
use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
enum IngestError {
    #[error("io error at {path}")]
    Io { path: PathBuf, #[source] source: std::io::Error },

    #[error("{path} is not a supported image (found magic {magic:02x?})")]
    NotAnImage { path: PathBuf, magic: [u8; 4] },

    #[error("destination {0} already exists")]
    Collision(PathBuf),
}

fn read(path: &Path) -> Result<Vec<u8>, IngestError> {
    std::fs::read(path).map_err(|source| IngestError::Io {
        path: path.to_path_buf(),
        source,
    })
}
```

`?` propagates and auto-converts via `From`. `thiserror` generates that `From` for you.

**The rule for this codebase:** `thiserror` (typed enums) in library/core code so callers can
*match* on the failure; `anyhow::Result` only at the top-level binary/CLI where you just print.
Never `unwrap()` or `expect()` on a path that touches a user's files.

Useful combinators — learn these so you stop writing `match` pyramids:

```rust
opt.map(f).unwrap_or(default)
opt.unwrap_or_else(|| expensive())
opt.ok_or(Error::Missing)?          // Option -> Result
res.ok()                            // Result -> Option
res.map_err(Error::from)?
opt.and_then(|x| x.checked_sub(1))
```

---

## Block 3 — Iterators, Closures, Collections (1.5h)

Iterators replace ~80% of loops you'd write in another language, and they're lazy — nothing
runs until a consuming adapter is called. For heavy disk work, laziness is what keeps you
from loading 40 GB of paths into a `Vec`.

```rust
let total: u64 = files.iter().filter(|f| f.is_raw()).map(|f| f.bytes).sum();

let by_ext: HashMap<&str, Vec<&ImageFile>> = files.iter()
    .filter_map(|f| f.path.extension()?.to_str().map(|e| (e, f)))
    .fold(HashMap::new(), |mut acc, (e, f)| {
        acc.entry(e).or_default().push(f);
        acc
    });

// Fail fast: Result<Vec<_>, E> — stops at the first error
let loaded: Result<Vec<_>, _> = paths.iter().map(|p| read(p)).collect();

// Or partition successes from failures
let (ok, bad): (Vec<_>, Vec<_>) = results.into_iter().partition(Result::is_ok);
```

### `iter()` vs `iter_mut()` vs `into_iter()`

| Call | Yields | Collection after |
|---|---|---|
| `.iter()` | `&T` | still usable |
| `.iter_mut()` | `&mut T` | still usable |
| `.into_iter()` | `T` | consumed |

### Closures

```rust
let factor = 2;
let f = move |x: u64| x * factor;    // `move` captures by value — required for threads
```

Three traits, in order of restrictiveness:
- `Fn` — captures by `&`, callable many times
- `FnMut` — captures by `&mut`, callable many times, mutates state
- `FnOnce` — consumes captures, callable once

Take `impl Fn(...)` in a signature when you can; `Box<dyn Fn(...) + Send + Sync>` when it must be stored.

### Collections you'll reach for

`Vec`, `HashMap`, `HashSet`, `BTreeMap` (sorted — good for deterministic output),
`VecDeque` (queue), `BinaryHeap` (priority scheduling).

### Drill (40 min)

`rustlings` sections: `options`, `error_handling`, `generics`, `traits`, `iterators`, `hashmaps`.

---

## Block 4 — Smart Pointers & Concurrency (2h)

### The pointer zoo — pick by question

| Question | Answer |
|---|---|
| Heap allocation / recursive type / `dyn Trait` | `Box<T>` |
| Multiple owners, single thread | `Rc<T>` |
| Multiple owners, across threads | `Arc<T>` |
| Mutate through a shared ref, single thread | `RefCell<T>` (panics on violation at runtime) |
| Mutate through a shared ref, across threads | `Mutex<T>` / `RwLock<T>` |
| Shared mutable across threads | `Arc<Mutex<T>>` |

### `Send` and `Sync`

- `Send` = safe to **move** to another thread.
- `Sync` = safe to **share by reference** across threads (`&T` is `Send`).

Auto-derived. When the compiler says "`Rc<T>` cannot be sent between threads safely," the
fix is `Arc`. When it says a closure isn't `Send`, something non-thread-safe got captured.

### Threads

```rust
use std::sync::{Arc, Mutex};
use std::sync::mpsc;

// Scoped threads — can borrow from the stack, no 'static requirement, auto-joined
std::thread::scope(|s| {
    for chunk in files.chunks(100) {
        s.spawn(move || process(chunk));
    }
});

// Channels — the preferred way to move data between threads
let (tx, rx) = mpsc::channel::<Progress>();
std::thread::spawn(move || {
    tx.send(Progress::Done(path)).ok();
});
for msg in rx { update_ui(msg); }
```

> **Share memory by communicating, not the other way round.** Prefer a channel over
> `Arc<Mutex<Vec<_>>>` wherever the data flows one direction.

### Rayon — the 10-second parallel win

```rust
use rayon::prelude::*;

let results: Vec<_> = files.par_iter()
    .map(|f| validate(f))
    .collect();                 // order preserved, thread pool managed for you
```

For CPU-bound work over a collection, `par_iter()` is almost always the right answer and
almost always all you need. Reach for raw threads only when you need long-lived workers.

### Async — do you need it?

**For this project: probably not, or only at the edges.** Async buys you *concurrency for
waiting* (thousands of network sockets). Your workload is disk + CPU bound, where blocking
threads and rayon are simpler and usually faster. Learn `tokio` only if:
- you add a network/socket IPC layer, or
- your GUI framework demands it (Tauri does — but only for its command handlers).

If you do: `async fn`, `.await`, `tokio::spawn`, `tokio::sync::{mpsc, Semaphore}`,
`spawn_blocking` for CPU/disk work. The one rule: **never block inside an async task.**

### Drill (45 min)

`rustlings`: `smart_pointers`, `threads`. Then write it yourself: spawn 4 threads that each
hash a slice of files and report over a channel to a main-thread progress printer.

---

## Block 5 — The Project Concepts (2h)

Now map the language onto what the project actually has to do. This is the part that
turns revision into a head start.

### 5.1 Local-first processing

Concepts: module layout, `Path`/`PathBuf`, filesystem traversal.

```rust
// src/main.rs
mod scan;      // -> src/scan.rs or src/scan/mod.rs
mod validate;
mod store;
use scan::Scanner;    // `pub` controls what escapes a module
```

**Never build paths with string concatenation** — it breaks on Windows:

```rust
let out = root.join("previews").join(format!("{stem}.jpg"));   // correct
```

Crates: `walkdir` (recursive traversal with symlink/depth control), `dirs` (per-OS config,
cache, and data dirs).

### 5.2 Heavy disk I/O

Concepts: `Read`/`Write`/`Seek` traits, buffering, streaming instead of slurping.

```rust
use std::io::{BufReader, Read};

// WRONG for large files: fs::read() pulls the whole file into RAM.
// Right: stream in fixed chunks with a reusable buffer.
let mut r = BufReader::with_capacity(64 * 1024, File::open(path)?);
let mut buf = [0u8; 64 * 1024];
loop {
    let n = r.read(&mut buf)?;
    if n == 0 { break; }
    hasher.update(&buf[..n]);
}
```

Always wrap `File` in `BufReader`/`BufWriter` — an unbuffered `File` syscalls per byte.
Use `metadata()` to get size without opening. Consider `memmap2` for random access into
large RAW files, understanding the aliasing/UB caveat (the file must not be mutated
underneath you).

### 5.3 Bounded resource usage

Concepts: thread pools, bounded channels, semaphores, backpressure, streaming iterators.

The failure mode you're guarding against is: user points the tool at 80,000 photos, you
`collect()` everything, and the process gets OOM-killed.

```rust
// Bounded channel = backpressure. Producer blocks when consumers fall behind.
let (tx, rx) = crossbeam_channel::bounded::<PathBuf>(256);

// Cap the pool explicitly rather than defaulting to core count
rayon::ThreadPoolBuilder::new().num_threads(4).build_global()?;

// Cap decoded-pixel memory, not just file bytes — a 40 MP JPEG is ~120 MB decoded
const MAX_DECODED_BYTES: u64 = 512 * 1024 * 1024;
```

Rules: prefer iterators over `Vec` for pipelines; bound *every* queue; cap concurrent open
file handles (OSes have limits, macOS's is low by default); decide the memory ceiling
up front and enforce it, don't hope.

### 5.4 Persistence and recovery

Concepts: `serde`, atomic writes, idempotency, `Drop`.

**Atomic write is the single most important pattern here.** A partially-written manifest
after a crash is worse than no manifest.

```rust
use std::io::Write;

fn write_atomic(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let dir = path.parent().expect("path has a parent");
    let mut tmp = tempfile::NamedTempFile::new_in(dir)?;   // same filesystem — rename must be atomic
    tmp.write_all(bytes)?;
    tmp.as_file().sync_all()?;                             // force to disk BEFORE the rename
    tmp.persist(path)?;                                    // atomic rename
    Ok(())
}
```

For job state, prefer **SQLite (`rusqlite`) over a JSON blob**: transactions, no
rewrite-the-world on every update, crash-safe by construction. Model each job so re-running
it is idempotent — record `(source_hash, dest, state)` and skip anything already `Done`.

`Drop` runs on scope exit but **not on panic-abort or process kill**, so never rely on it
for durability. Durability comes from `fsync` + rename, not from destructors.

### 5.5 Concurrent jobs

Concepts: `Send`/`Sync`, channels, worker pools, cancellation.

```rust
// Cancellation: a shared flag workers check between units of work
use std::sync::atomic::{AtomicBool, Ordering};
let cancel = Arc::new(AtomicBool::new(false));
if cancel.load(Ordering::Relaxed) { return Ok(Outcome::Cancelled); }
```

Structure: one scanner thread feeding a bounded channel → N workers → one collector thread
owning the DB writes. A single writer avoids lock contention and makes ordering reasonable.

### 5.6 IPC boundaries (if you add a client)

Concepts: serde as wire format, typed protocol enums, framing, versioning.

```rust
#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
enum Request {
    StartScan { root: PathBuf, recursive: bool },
    Cancel { job_id: u64 },
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
enum Response {
    Accepted { job_id: u64 },
    Progress { job_id: u64, done: u32, total: u32 },
    Error { code: ErrorCode, message: String },    // never leak a raw Debug string
}
```

Rules: the boundary is a **trust boundary** — validate everything crossing it, especially
paths (reject `..` traversal, canonicalize and check the result is under an allowed root).
Length-prefix your frames if you're on a raw socket. Version the protocol from day one.
If you use Tauri, its `#[tauri::command]` handles the transport; you still own validation.

### 5.7 Image validation

Concepts: byte-level parsing, custom errors, never trusting user input.

**A file extension is a claim, not a fact.** Read magic bytes:

```rust
fn sniff(bytes: &[u8]) -> Option<Format> {
    match bytes {
        [0xFF, 0xD8, 0xFF, ..]                       => Some(Format::Jpeg),
        [0x89, b'P', b'N', b'G', 0x0D, 0x0A, ..]     => Some(Format::Png),
        [b'I', b'I', 0x2A, 0x00, ..]                 => Some(Format::Tiff), // CR2/NEF/ARW are TIFF-based
        [b'M', b'M', 0x00, 0x2A, ..]                 => Some(Format::Tiff),
        _ => None,
    }
}
```

Then guard the decode itself: enforce max dimensions **before** allocating (`image` exposes
`ImageDecoder::dimensions()` and `Limits`), so a 100,000 × 100,000 px "decompression bomb"
is rejected rather than allocating 40 GB. Treat a decode panic as a real risk — the
`image` crate's pure-Rust decoders are memory-safe but can still abort on OOM.

Crates: `infer` or `imagesize` for cheap sniffing, `image` for decode, `kamadak-exif` for EXIF,
`rawler`/`libraw` bindings if you need real RAW support.

### 5.8 Cross-platform desktop

```rust
#[cfg(target_os = "windows")]
fn lock(f: &File) -> io::Result<()> { /* ... */ }

#[cfg(unix)]
fn lock(f: &File) -> io::Result<()> { /* ... */ }

if cfg!(windows) { /* runtime branch */ }
```

The real landmines:
- **Case sensitivity** — macOS/Windows are usually case-insensitive, Linux isn't. `Photo.JPG` and `photo.jpg` may be the same file.
- **Path length** — Windows historically caps at 260 chars; use `\\?\` prefixed paths or `dunce`.
- **Non-UTF-8 paths** — `Path` is `OsStr`, not `String`. `.to_str()` returns `Option`. Use `.display()` for output and `.to_string_lossy()` when you must have a string, and never round-trip a path through `String`.
- **File locking / open handles** — Windows won't let you delete or rename an open file; Unix will.
- **Where files go** — use `dirs::data_dir()`, `cache_dir()`, `config_dir()`; never hardcode `~/.config`.
- **Trailing separators, UNC paths, reserved names** (`CON`, `NUL`, `PRN` on Windows).

Cross-compile checks: `cargo check --target x86_64-pc-windows-msvc` etc., in CI.

### 5.9 A failure mode with genuine consequences

You are moving, renaming, or deleting **irreplaceable photographs**. Design for that:

| Guard | Implementation |
|---|---|
| Never hard-delete | `trash` crate — send to OS recycle bin |
| Prove it copied before removing source | verify with `blake3` hash of the destination |
| Dry run by default | `--dry-run` flag that logs the full plan and mutates nothing |
| Never silently overwrite | check `dest.exists()` first; `fs::rename` clobbers on Unix |
| Same-filesystem assumption | `fs::rename` fails across devices — fall back to copy+verify+delete |
| Auditability | append-only log of every mutation, written before it happens |
| No panics on user paths | ban `unwrap`/`expect` in these modules |

```rust
// Enforce it in the crate root:
#![cfg_attr(not(test), deny(clippy::unwrap_used, clippy::expect_used))]
```

Also: handle `Ctrl-C` (`ctrlc` crate) to finish the in-flight operation and flush state
rather than dying mid-write.

---

## The three exercises (do these — they *are* the project in miniature)

If you only do one thing after the reading, do these. Each builds on the last.

**1. Typed scanner (~45 min)** — walk a directory recursively, read the first 16 bytes of
each file, classify by magic number, and print a count per real format. Report errors
per-file without aborting the whole scan (`Vec<Result<_, _>>`, not `?` at the top).
*Exercises: `Path`, iterators, custom error enum, `walkdir`.*

**2. Bounded parallel hasher (~60 min)** — take the scanner's output, hash every image with
`blake3` across a rayon pool capped at 4 threads, stream chunks through a fixed 64 KB
buffer, and report progress over a bounded channel to a printer on the main thread. Add a
`Ctrl-C` handler that cancels cleanly.
*Exercises: `Arc`, `Send`/`Sync`, channels, backpressure, streaming I/O, cancellation.*

**3. Crash-safe manifest (~60 min)** — persist `(path, hash, format, state)` to SQLite in a
transaction. Kill the process mid-run (`kill -9`). On restart, resume without re-hashing
anything already recorded. Then do the same with atomic JSON writes and observe why SQLite
is less work.
*Exercises: serde, atomic rename + fsync, idempotency, recovery.*

---

## Self-check — can you answer these without looking?

1. Why does `String` not implement `Copy`?
2. When does `&mut T` coexist with `&T`? (Never — why does that rule prevent data races?)
3. `Rc` vs `Arc` — what's the actual cost difference, and why can't `Rc` cross threads?
4. What does `?` desugar to, and what trait makes it convert error types?
5. `.iter()` vs `.into_iter()` on a `Vec<String>` — what can you do after each?
6. Why is `write to temp + fsync + rename` atomic, and why must the temp file be on the same filesystem?
7. Why is `fs::read()` the wrong call for a 60 MB RAW file in a 40,000-file loop?
8. What does a *bounded* channel give you that an unbounded one doesn't?
9. Why can't you `.to_str().unwrap()` a `Path`?
10. Given `Box<dyn Trait>` vs `impl Trait` — when is each required?

Anything you can't answer, go back to that block.

---

## Crate shortlist

```toml
[dependencies]
# errors
thiserror = "2"
anyhow = "1"            # binary only

# fs + paths
walkdir = "2"
dirs = "5"
tempfile = "3"
trash = "5"

# concurrency
rayon = "1"
crossbeam-channel = "0.5"
ctrlc = "3"

# persistence
serde = { version = "1", features = ["derive"] }
serde_json = "1"
rusqlite = { version = "0.32", features = ["bundled"] }

# images
image = "0.25"
infer = "0.16"
kamadak-exif = "0.5"
blake3 = "1"

# observability
tracing = "0.1"
tracing-subscriber = "0.3"
```

Check current versions with `cargo add <crate>` rather than trusting these numbers.

---

## Reference material (skim, don't read cover to cover)

| Resource | Use it for |
|---|---|
| The Book, ch. 4, 10, 13, 15, 16 | Ownership, traits/lifetimes, iterators, smart pointers, concurrency |
| Rust by Example | Fast runnable snippets when you forget syntax |
| `std` docs (`cargo doc --open`) | `std::fs`, `std::io`, `std::path`, `std::sync` — read these three modules' overviews properly |
| Rust API Guidelines | Naming and signature conventions, so your code reads like Rust |
| `cargo clippy` | The best teacher you have. Fix every lint. |

**Suggested day:** Blocks 1–4 in the morning and early afternoon (~7h with breaks), Block 5
and Exercise 1 in the evening. Exercises 2 and 3 spill into day two — and by then you're
writing the actual project, which is the point.
