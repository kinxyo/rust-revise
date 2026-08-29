# Current Goal

**A recursive directory scanner that classifies files by their magic bytes.** Not a toy — it's stage one of the real project, so you keep it.

Filesystem code is the ideal Rust teacher because it's saturated with exactly the things Rust makes you decide:

- Every path is PathBuf or &Path — you cannot dodge the owned/borrowed choice
- Everything returns Option (a path may have no extension, no filename, no parent)
- Everything returns Result (permission denied, broken symlink, file vanished mid-scan)
- It naturally grows into threads, and then into persistence

Do it in six stages

Each is 30–60 minutes and each one forces a specific concept by making it unavoidable — not by an exercise telling you to use it.

1. Walk and print. cargo run -- /some/dir → print every file path found, recursively. Write the recursion yourself with fs::read_dir before reaching for walkdir.
→ forces: PathBuf vs &Path, Result, ?, iterators, why .display() exists

2. Sniff the type. Open each file, read the first 16 bytes, classify by magic number, print path → format.
→ forces: stack buffers, File/Read, short reads, slice patterns, Option, a data-carrying enum

3. Aggregate. Build a Vec<ImageFile>, then a count per format.
→ forces: struct ownership, moving vs borrowing in loops, HashMap, .iter() vs .into_iter() — this stage is where borrow errors start, which is the point

4. Don't die on one bad file. A permission error on file 3,000 must not abort the scan. Report it and continue.
→ forces: a thiserror enum, error context, Result i top is wrong here

5. Parallelize. Swap the loop for rayon's par_iter(
→ forces: Send/Sync, Arc, and the first time the compiler stops a race you didn't see

6. Persist. Write results to JSON with serde, using write-temp + fsync + rename. kill -9 it mid-run and confirm the file is either
complete or absent, never half-written.
→ forces: serde, Drop, atomic writes, the durability model

Don't .clone() to escape a borrow error. When the compiler stops you, sit with it for a minute — the fix is almost always "restructure so the borrow is shorter" or "own it deliberately." Cloning past errors is how people end up writing Rust for a year while still finding it confusing.

Also run cargo clippy after every stage. It will tell you the idiomatic form of what you just wrote, which is faster feedback than any book.
