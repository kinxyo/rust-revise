# Rust Notes

## File I/O: `fs::` vs `File`/`OpenOptions`

**`fs::` functions** — you touch the file once and it fits in memory.
**`File`/`OpenOptions`** — you need to keep the handle.

The `fs::` helpers are just the `File` versions pre-composed:
- `fs::read(p)` = `File::open` + `read_to_end`
- `fs::write(p, data)` = `File::create` + `write_all`

Nothing is hidden — you just can't hold the handle afterward.

### Decision table

| Situation | Use |
|---|---|
| Read a whole config/text file | `fs::read_to_string(p)?` |
| Read a whole file as bytes | `fs::read(p)?` |
| Write a whole file, replacing it | `fs::write(p, data)?` |
| **Append** to a file | `OpenOptions::new().append(true).create(true)` |
| File might be huge (RAW photo, video) | `File` + `BufReader` + chunk loop |
| Many writes in a loop | `File` + `BufWriter` |
| Write then read back the same handle | `File` + `Seek` |
| Create only if absent (lockfile) | `OpenOptions::new().create_new(true)` |
| Just checking size / existence | `fs::metadata(p)?` — doesn't open at all |

### The three things that force `OpenOptions`

Everything else is convenience. These are real capability gaps:

1. **Appending** — `fs::write` truncates. There is no `fs::append`.
2. **Create-if-absent, atomically** — `create_new(true)` fails with `AlreadyExists`
   instead of clobbering. Check + create in one syscall, so two processes can't both win.
3. **Keeping the handle open** — streaming, seeking, or holding it for the program's life.

### Why this matters for the scanner

```rust
let mut f = File::open(path)?;
let mut magic = [0u8; 16];
let n = f.read(&mut magic)?;      // 16 bytes, not 30 MB
```

`fs::read` on a RAW file pulls all 30 MB into memory just to look at the first 4 bytes.
Across 40,000 photos that's ~1.2 TB of pointless I/O.

---

## Owned vs borrowed

| Owned (has a heap buffer, frees it) | Borrowed (a view, frees nothing) |
|---|---|
| `String` | `&str` |
| `Vec<T>` | `&[T]` |
| `PathBuf` | `&Path` |
| `OsString` | `&OsStr` |

**Rule: take the borrowed form in arguments, return the owned form.**

```rust
fn stem(p: &Path) -> String     // ✓ every caller works, nobody is forced to allocate
fn stem(p: PathBuf) -> String   // ✗ forces callers to give up their path
```

The distinction is **owned vs borrowed**, not heap vs stack. A `&str` can point at
`.rodata` (a literal), at the heap (inside a `String`), or at the stack. The type says
"someone else owns these bytes," not where they live.

Free conversions (no allocation, same buffer):
```rust
s.as_str()        // String -> &str
s.as_bytes()      // String -> &[u8]
s.into_bytes()    // String -> Vec<u8>  (consumes)
String::from_utf8(bytes)?   // Vec<u8> -> String, validates in place
```

`Vec<u8>` and `String` have identical layout. `String` is a `Vec<u8>` plus the guarantee
that the bytes are valid UTF-8.

---

## Moves — what actually happens

Returning a `String` from a function does **not** copy the string. Proven with pointers:

```
inside make():   heap ptr = 0x596a21ec1d50
back in main():  heap ptr = 0x596a21ec1d50   <- same allocation
after clone():   heap ptr = 0x596a21ec1d70   <- different (this is a real copy)
```

**The buffer was never inside the function.** `String::from` allocates on the heap
immediately; the stack frame only ever held a 24-byte `{ptr, cap, len}` handle. Returning
copies those 24 bytes (often elided by RVO). The heap is untouched.

A move is two things, and only the second is real work:
1. copy the handle (24 bytes, usually elided)
2. **mark the source dead so `Drop` doesn't run on it** ← this is the whole point

One handle, one free, guaranteed. It's a type-system rule enforced on MIR — LLVM never
sees a "move," just a memcpy and the *absence* of a destructor call.

There is no third memory strategy. Either the caller supplies the buffer, or it's a heap
allocation. Returning owned **is** the heap allocation — what Rust adds is that the
free-obligation rides in the type instead of a doc comment.

Rust never promotes stack to heap automatically. `fn f() -> [u8; 1024]` copies 1024 bytes.
If you want heap, name a heap type.

---

## When to care about performance

Measured, 1,000,000 iterations, `-O`:

| Approach | Allocations | Time | Per iter |
|---|---|---|---|
| return owned `String` (move) | 1,000,000 | 18.2 ms | 18.2 ns |
| reused caller-provided buffer | **1** | 1.66 ms | 1.7 ns |
| return + `.trim().to_string()` | 2,000,000 | 28.0 ms | 28.0 ns |

Returning added **zero** allocations. `.to_string()` is what doubles them.

### Cost anchors

| Operation | Cost | vs one malloc |
|---|---|---|
| L1 cache hit | 1 ns | 0.05× |
| **malloc + free** | **~17 ns** | 1× |
| syscall | ~500 ns | 30× |
| 4 KB random SSD read | ~100 µs | 6,000× |
| read a 30 MB RAW file | ~30 ms | 1,800,000× |
| human types at a prompt | ~2 s | 120,000,000× |

### The rule

1. Runs under ~100k times? Stop thinking about it.
2. Is there I/O in the same loop? The I/O is 1,000–1,000,000× bigger.
3. Haven't profiled? You're guessing, and you'll guess wrong.

Reuse buffers when the loop count is large **and** the buffer is large. Small allocations
in small loops are free. Never allocate inside a per-pixel loop (40 MP = 40M iterations).

---

## Enums

```rust
enum State {
    Queued,                                  // unit
    Done(PathBuf),                           // tuple
    Failed { path: PathBuf, msg: String },   // struct
}
```

Size = largest variant + discriminant tag. A unit-only enum is 1 byte.

**Match exhaustively; resist `_ => {}`.** With no catch-all, adding a variant makes every
match fail to compile until you handle it — the compiler hands you a to-do list. A `_` arm
silently swallows new variants forever.

Patterns nest, so no inner match is needed:
```rust
match choice {
    Some(MenuOptions::Create) => create_file(),
    None => println!("invalid"),
}
```

### Display vs Debug

| | `Debug` | `Display` |
|---|---|---|
| Spec | `{:?}` | `{}` |
| For | you, debugging | your user |
| Derivable | **yes** | **never** |

`#[derive(Debug)]` prints the variant name — that's free. `Display` needs a hand-written
match because "Create" → "Create a file" only exists in your head.

```rust
impl fmt::Display for MenuOptions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self { Self::Create => "Create a file", ... };
        f.pad(s)          // NOT write!(f, "{s}") — see traps
    }
}
```

Implementing `Display` gives you `.to_string()` free (blanket `impl<T: Display> ToString`).

`Option<T>` implements `Debug` but **not** `Display` — there's no universal way to show
"nothing" to a user. Unwrap it yourself.

---

## Traps hit so far

**`error[E0716]: temporary value dropped while borrowed`** — the most common borrow error.
```rust
let f = prompt("file?").as_str();   // ✗ the String is a temporary, dies at the ;
let s = prompt("file?")?;           // ✓ name it first
let f = s.as_str();
```
Never chain a borrow off a function that returns owned data.

**Deref coercion does NOT apply in match patterns.**
```rust
match some_string { "1" => ... }              // ✗ String vs &str
match some_string.as_str() { "1" => ... }     // ✓
```
It *does* apply to function arguments and method calls — that's why `f(&string)` works
where `f` takes `&str`.

**Trait methods are invisible until the trait is imported.** "method not found" on a
`File` almost always means a missing `use std::io::{Read, Write, Seek}`.

**`.create(true)` needs `.write(true)`.** Runtime error, not compile error:
`"creating or truncating a file requires write or append access"`.

**`write` vs `write_all`.** `write` may do a partial write and returns how many bytes
landed. Ignoring that silently loses data. Use `write_all` unless you're handling short
writes yourself. Same shape as `read` returning `n` — always slice `&buf[..n]`.

**`f.pad(s)` vs `write!(f, "{s}")` in a `Display` impl.** `write!` ignores the formatter's
width/alignment, so `{:>20}` silently does nothing. `f.pad` honors them.

**`read_line` appends, it doesn't replace.** Reusing a `String` in a loop requires
`.clear()` first. It also keeps the trailing `\n`.

**`let _guard = X;` vs `let _ = X;`** — binding to bare `_` drops the value *immediately*.
Guards must have a real name.

---

## CLI args

```rust
std::env::args()      // Iterator<Item = String>     PANICS on invalid UTF-8
std::env::args_os()   // Iterator<Item = OsString>   never panics
```

`args[0]` is the program path, not your first argument.

**Use `args_os()` for anything that's a path** — paths are arbitrary bytes on Linux, and
`args()` will panic on a folder whose name isn't valid UTF-8.

```rust
let root: PathBuf = std::env::args_os().nth(1)
    .map(PathBuf::from)
    .unwrap_or_else(|| PathBuf::from("."));
```

Switch to `clap` (derive feature) the moment you want a second flag. `PathBuf` works as a
field type and clap avoids the UTF-8 panic internally.

---

## User input

```rust
fn prompt(msg: &str) -> io::Result<String> {
    print!("{msg}");
    io::stdout().flush()?;              // REQUIRED — see below
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}
```

**Flush only when you need bytes visible before you block.** A prompt with no newline sits
in the buffer while `read_line` waits, so the user sees nothing. Everywhere else, letting
`println!` flush on its newline is fine and batches the writes into one syscall.

Also flush before `process::exit()` — it skips destructors and can drop buffered output.

Reading in a loop: take `&mut String` and `.clear()` it instead of returning a new one.
One allocation total instead of one per line.

---

## Terminal control

Escape codes:

| Code | Effect |
|---|---|
| `\x1b[2J` | clear screen |
| `\x1b[H` | cursor home |
| `\x1b[K` | clear to end of line |
| `\x1b[?25l` / `\x1b[?25h` | hide / show cursor |
| `\x1b[?1049h` / `\x1b[?1049l` | enter / leave alt screen |

Use the **alt screen** for a TUI rather than clearing — it preserves the user's scrollback
and restores it on exit. That's what vim and less do.

### Raw mode

Line buffering means nothing reaches you until Enter. Raw mode fixes that. Use `crossterm`:

```rust
crossterm::terminal::enable_raw_mode()?;
crossterm::terminal::disable_raw_mode()?;
```

Over raw `libc`/termios because (a) Windows has no termios — it's `SetConsoleMode`, a
different API entirely — and (b) arrow keys arrive as multi-byte escape sequences
(`\x1b[A`) that crossterm decodes into `KeyCode::Up` for you.

What raw mode changes: no echo, no line buffering, **Ctrl-C no longer sends SIGINT**, and
`\n` no longer implies a carriage return — so `println!` staircases unless you use
crossterm's print helpers or emit `\r\n`.

### The guard is not optional

```rust
struct TermGuard;
impl Drop for TermGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        print!("\x1b[?1049l");
        let _ = io::stdout().flush();
    }
}
```

Runs on normal return **and** on panic-unwind. Does **not** run on `process::exit` —
which is why a menu's "Exit" option must return a `Flow::Quit` signal and let `main`
return normally, rather than calling `exit()`.

---

## FFI with `libc`

`cargo add libc`. It's raw C declarations — no abstraction, no safety. **Lookup rule:
`man 3 tcsetattr` gives the signature, `libc::tcsetattr` is that same function.** 1:1 mirror,
including constant names (`libc::ICANON`, `libc::O_RDONLY`, `libc::EAGAIN`).

### The call pattern

Every call is `unsafe`; C's error convention is `-1` + `errno`:

```rust
let ret = unsafe { libc::tcsetattr(fd, libc::TCSANOW, &raw) };
if ret == -1 {
    return Err(io::Error::last_os_error());   // reads errno, wraps it
}
```

`io::Error::last_os_error()` is the bridge from `errno` to a real Rust error with a
matchable `.kind()`. Keep `unsafe` blocks as tight as possible and expose a safe wrapper —
callers should never see `unsafe`.

### Getting a file descriptor

```rust
use std::os::fd::AsRawFd;
let fd = io::stdin().as_raw_fd();      // 0; borrows, File still closes on drop
                                       // into_raw_fd() transfers ownership — you must close
```

### Gotchas

- **Strings need a NUL.** Rust strings aren't NUL-terminated. Use `CString::new(s)?` then
  `.as_ptr()`. Never pass `&str::as_ptr()` to C.
- **Platform-gate non-portable items.** `libc::termios` doesn't exist on Windows:
  `#[cfg(unix)] fn cbreak_on() -> io::Result<libc::termios> { ... }`
- **`MaybeUninit` is correct here.** `tcgetattr` *fills* the struct, so zeroing first is
  wasted work and `termios` has no meaningful default. This is the output-parameter case
  the type exists for.

---

## Bitfields

```rust
raw.c_lflag &= !(libc::ICANON | libc::ECHO);
```

`c_lflag` is a `u32` where each bit is a flag:

```
ICANON            = 0x2          ...00000010     one bit
ECHO              = 0x8          ...00001000     one bit
ICANON | ECHO     = 0xa          ...00001010     both
!(ICANON | ECHO)  = 0xfffffff5   ...11110101     everything EXCEPT those two

c_lflag before      0x8a3b       ...00111011
c_lflag &= mask  →  0x8a31       ...00110001     those two cleared, rest untouched
```

Means: **turn these flags off, leave every other flag alone.** The mask dance (rather than
plain assignment) exists because `c_lflag` holds a dozen other flags you didn't set.

**`!` is bitwise NOT in Rust.** C uses `~`; Rust has no `~` at all and overloads `!` by
type — `!true == false`, `!0u8 == 255`.

```rust
x |=  FLAG;              // set
x &= !FLAG;              // clear
x ^=  FLAG;              // toggle
if x & FLAG != 0 { }     // test
```

(Zig exposes `termios.lflag` as a packed struct of named bools, so you skip this. The
`libc` crate is a straight C mirror, so you do the masking yourself.)

---

## cbreak vs raw mode

**You usually want cbreak, not full raw mode.** For "read one keypress," clear two flags:

```
ICANON off  →  bytes arrive immediately, no line buffering
ECHO   off  →  don't print what the user typed
```

Leave the rest and you keep:

| Kept | Because |
|---|---|
| `ISIG` | Ctrl-C still kills the program — you don't have to handle it |
| `OPOST` | `println!` doesn't staircase (still emits `\r\n`) |
| `ICRNL` | Enter still delivers `\n`, so `read_line` elsewhere is unaffected |

`crossterm::enable_raw_mode()` clears ~8 flags — right for a full TUI, overkill for a menu.

### Enter sends `\r`, not `\n`

The keyboard has **always** sent `\r` (0x0D). `ICRNL` is the terminal driver translating it
to `\n` for you. Full raw mode turns that off, so `read_line` (which waits for `\n`) blocks
forever — and with `ECHO` and `ISIG` also off, you're typing invisibly into a hang you
can't Ctrl-C out of.

Same trap exists in C (`fgets`) and Zig (`readUntilDelimiter`). It's termios, not Rust.

### Working example

```rust
fn cbreak_on() -> io::Result<libc::termios> {
    let fd = io::stdin().as_raw_fd();
    let mut orig = MaybeUninit::<libc::termios>::uninit();
    if unsafe { libc::tcgetattr(fd, orig.as_mut_ptr()) } != 0 {
        return Err(io::Error::last_os_error());
    }
    let orig = unsafe { orig.assume_init() };

    let mut raw = orig;
    raw.c_lflag &= !(libc::ICANON | libc::ECHO);
    raw.c_cc[libc::VMIN] = 1;      // return after 1 byte
    raw.c_cc[libc::VTIME] = 0;     // no timeout

    if unsafe { libc::tcsetattr(fd, libc::TCSANOW, &raw) } != 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(orig)
}

fn read_key() -> io::Result<u8> {
    let orig = cbreak_on()?;
    let mut buf = [0u8; 4];
    let result = io::stdin().read(&mut buf);
    cbreak_off(&orig)?;            // restore BEFORE propagating
    let n = result?;
    Ok(if n == 0 { 0 } else { buf[0] })
}
```

**Restore before the `?`.** Capture the `Result`, restore the terminal, *then* propagate —
otherwise a failed read leaves the user with no echo.

**Use `read`, not `read_exact`, with a >1-byte buffer.** `read_exact([0u8; 4])` blocks until
4 bytes arrive, so one keypress hangs. Also: Rust's Windows console stdin rejects buffers
smaller than 4 bytes (UTF-16 → UTF-8 re-encoding), so `[0u8; 4]` + `read` is the portable
form.

**`ENOTTY` when stdin is piped.** `tcgetattr` fails if stdin isn't a terminal. Guard with
`unsafe { libc::isatty(fd) } != 0` and fall back to `read_line` if you want piping to work.

### Escape codes it doesn't cover

Single-byte keys (digits, Esc=27, letters) are portable. Arrow keys are `\x1b[A` etc. on
Unix and a different encoding on Windows — that's where `crossterm::event::read()` earns
its keep by normalizing both into `KeyCode::Up`.

---

## Capture → restore → propagate

Any time you've changed **global state** (terminal mode, cwd, env var, signal handler) and
the next call can fail, the restore must sit *between* the call and the `?`:

```rust
let orig = cbreak_on()?;

let result = io::stdin().read(&mut buf);   // no ? — hold the Result
cbreak_off(&orig)?;                        // restore ALWAYS runs
let n = result?;                           // now propagate
```

Because this silently breaks:

```rust
let orig = cbreak_on()?;
let n = io::stdin().read(&mut buf)?;       // ✗ early return skips the restore
cbreak_off(&orig)?;                        //   user is left with no echo
```

### The better fix: a Drop guard

Manual capture/restore is fine for a one-off. For anything used more than once, make the
restore automatic — then `?` is safe everywhere and it survives panics too:

```rust
struct Cbreak(libc::termios);

impl Cbreak {
    fn on() -> io::Result<Self> { Ok(Cbreak(cbreak_on()?)) }
}

impl Drop for Cbreak {
    fn drop(&mut self) { let _ = cbreak_off(&self.0); }
}

// usage
let _guard = Cbreak::on()?;
let n = io::stdin().read(&mut buf)?;       // ✓ guard restores on early return
```

Remember `let _guard = ...`, not `let _ = ...` — bare `_` drops immediately.

### Does this happen in Zig?

**No — `defer` handles it in one line:**

```zig
const orig = try cbreakOn();
defer cbreakOff(orig);
const n = try posix.read(fd, &buf);   // defer still runs on the error return
```

Rust has the same capability, spelled as a **type** instead of a **statement**:

| | Zig `defer` | Rust `Drop` |
|---|---|---|
| Ceremony | one line, per use | a struct + impl, once |
| Can you forget it? | yes — every call site | no, if the guard is the only way to get the resource |
| Runs on panic/unwind | no (Zig has no unwinding) | yes |

So the capture→restore→propagate dance is what you write *when you don't want to define a
guard type*. It's the manual version of `defer`. Define the guard as soon as the pattern
appears twice.
