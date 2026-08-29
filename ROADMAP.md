# Roadmap

Four projects, each a layer on the one below. Nothing is throwaway — every part becomes a
dependency of the next.

```
Part 4   Image processing        ← domain work, real algorithms
           uses ↓
Part 3   HTTP service            ← async, spawn_blocking, graceful shutdown
           wraps ↓
Part 2   Scanner engine          ← Send/Sync, Arc, channels, bounded pools
           reuses ↓
Part 1   CLI file manager        ← traits, Result, enums, FFI          ✓ done
```

The ordering is deliberate: **you can't learn `Send` before you understand ownership, and
you can't learn async before you understand threads.** Each part removes one source of
confusion before the next introduces it.

---

## Part 1 — CLI file manager ✓

Single-threaded, synchronous, one file at a time. No shared state, no concurrency.

Taught the language itself:

- Traits as the source of methods (`Read`, `Write`, `Display`) and why they must be imported
- `Result`, `?`, and holding a `Result` so cleanup runs before propagating
- Enums with data, exhaustive matching, `Option` at the parse boundary
- Moves — returning owned data without copying it
- `Drop` as scope-exit cleanup
- FFI: `libc` termios, `unsafe` at the boundary, `MaybeUninit` for output parameters

Code: `src/main.rs`. Notes: `notes.md`.

---

## Part 2 — Scanner engine

Point it at a directory. It walks the tree, opens each file, reads a small prefix, and
classifies by **magic bytes rather than extension** — surviving permission errors, files
that vanish mid-scan, and symlink loops.

Built as a library the later parts call, not a standalone demo.

| Stage | Build | Forces |
|---|---|---|
| 1 | Recursive walk, print paths | `Path`/`PathBuf`, `Option`, `?`, why `.display()` exists |
| 2 | Magic-byte classification | fixed buffers, short reads, slice patterns, data-carrying enums |
| 3 | Collect into records, aggregate | struct ownership, borrow-vs-move in loops, `HashMap` |
| 4 | Per-file error isolation | custom error enum, `Result` inside iterators, why `?` at the top is wrong |
| 5 | **Bounded parallel worker pool** | `Send`/`Sync`, `Arc`, channels, backpressure |
| 6 | Cooperative cancellation | `AtomicBool`, checking between work units |

Stages 1–4 are one long session. **Stage 5 is the point of the whole project** — that's
where `Send`, `Sync`, `Arc`, `Mutex`, and channels appear, with nothing else in the way.

Threads only. No async anywhere.

Target shape:

```rust
pub fn scan(root: &Path, opts: ScanOpts) -> Receiver<ScanEvent>
```

Streams progress, bounds its memory, doesn't die on bad files, can be cancelled.

Plan: `SCANNER_PLAN.md`.

---

## Part 3 — HTTP service

Wraps Part 2 in an API so scans run as background jobs. The scanner is already written and
tested, so **async is the only new variable.**

```
POST   /scans          start a scan, returns 202 + job id
GET    /scans/{id}     progress: state, files seen, bytes, errors
GET    /scans/{id}/files   the records found
DELETE /scans/{id}     request cancellation
```

| Stage | Build | Forces |
|---|---|---|
| 1 | axum router, `/health` | handlers, extractors, `IntoResponse`, serde derives |
| 2 | Job registry, `POST`/`GET /scans` | `async fn`, `tokio::spawn`, `Arc<RwLock<_>>`, why spawned futures need `Send + 'static` |
| 3 | Real scanner behind `spawn_blocking` | async vs blocking work, moving owned data across execution contexts |
| 4 | Cancellation + Ctrl-C shutdown | `CancellationToken`, task lifecycle, structured shutdown |

### The one rule that matters

**The scanner does not run in an async task.** Filesystem traversal and hashing are
blocking, CPU-bound work — running them on Tokio's async worker threads starves unrelated
requests. They go on a blocking thread and talk to the async side over a channel.

Getting that boundary right is most of what "learning async Rust" actually means.

### The other trap

Use `std::sync::RwLock` for the job registry, not `tokio::sync::RwLock`. The async locks
exist only for holding a lock *across* an `.await`, which a `HashMap` insert never does —
and they're slower. A `std::sync` guard held across an `.await` makes the future non-`Send`,
and the fix is always "shorten the critical section," never "switch to the async lock."

### Deliberately excluded

Auth, databases, Docker, OpenAPI generation, a web frontend, and — especially — hand-rolled
HTTP parsing. The job lifecycle *is* the project; everything else is distraction.

---

## Part 4 — Image processing

Not yet scoped. The likely direction is real image work on top of the scanner —
`opencv` and/or `imageproc` — where Part 2 supplies the file discovery and Part 3 supplies
the job control.

Concepts still uncovered after Parts 1–3, in rough priority order:

- **Persistence and recovery** — a resumable manifest (SQLite), atomic writes
  (temp + fsync + rename), idempotent re-runs
- **Destructive-operation discipline** — verify before replacing, dry-run by default,
  never silently overwrite
- **Cross-platform** — Part 1 went Unix-only on termios; anything shipped needs `#[cfg]`
  branches and Windows path handling
- **Real algorithms** — the actual domain work, rather than plumbing around it
