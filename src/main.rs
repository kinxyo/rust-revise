// INFO:
// CLI tool for basic file management.
// Learning/Revising project; Didn't bother with error handling much.

use std::{
    fmt,
    io::{self, Read, Result, Write},
    mem::MaybeUninit,
    os::fd::AsRawFd,
    process::Command,
};

fn main() -> Result<()> {
    cls();

    init()?;

    Ok(())
}

// ========= Main Fn(s) =========>

fn init() -> Result<()> {
    loop {
        let og_termios = cbreak_on()?;

        Menu::print_options();

        let mut buf = [0u8; 4];
        let result = io::stdin().read(&mut buf);

        cbreak_off(&og_termios)?;

        // The read's Result is held, not `?`d, until AFTER cbreak_off.
        // If we wrote `let n = io::stdin().read(&mut buf)?;` an I/O error would
        // return from init() immediately, skipping cbreak_off — leaving the user's
        // terminal with no echo and no line buffering, i.e. a broken shell.
        // Rule: when you've changed global state, restore it BETWEEN the fallible
        // call and the `?`.
        let n = result?;
        if n == 0 {
            return Ok(());
        };

        if buf[0] == b'q' {
            break;
        }

        exec_user_option(Menu::from_input(buf[0]))
    }
    Ok(())
}

fn exec_user_option(choice: Option<Menu>) {
    cls();

    match choice {
        Some(Menu::Create) => create_file(&prompt("file?")),
        Some(Menu::Read) => read_file(&prompt("file?")),
        Some(Menu::Update) => update_file(&prompt("file?")),
        Some(Menu::Delete) => delete_file(&prompt("file?")),
        None => println!("Next time, choose a NUMBER option from the menu :)"),
    }
}

// ========= Helper Fn(s) =========>

fn cls() {
    print!("\x1b[2J\x1b[H");
}

fn prompt(msg: &str) -> String {
    print!("{msg}: ");
    io::stdout()
        .flush()
        .expect("failed to flush stdout buffer.");

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("failed to read user input.");

    input.trim().to_string()
}

// ========= Enums =========>

enum Menu {
    Create,
    Read,
    Update,
    Delete,
}

impl Menu {
    fn print_options() {
        let list: [Menu; 4] = [Self::Create, Self::Read, Self::Update, Self::Delete];

        println!("Choose an option:\n");

        for (i, opt) in list.iter().enumerate() {
            println!("{}) {opt}", i + 1)
        }

        println!("\nq) Exit.\n");
    }

    fn from_input(input: u8) -> Option<Self> {
        match input {
            b'1' => Some(Self::Create),
            b'2' => Some(Self::Read),
            b'3' => Some(Self::Update),
            b'4' => Some(Self::Delete),
            _ => None,
        }
    }
}

impl fmt::Display for Menu {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Create => "Create a file.",
            Self::Read => "Read a file.",
            Self::Update => "Update a file.",
            Self::Delete => "Delete a file.",
        };

        f.pad(s)
    }
}

// ========= APIs -- File Management =========>

fn create_file(file: &str) {
    let mut fs_handle = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(file)
        .expect("Failed to create the file.");

    let content = prompt("what do you wnna write to it?");

    fs_handle
        .write_all(content.as_bytes())
        .expect("Failed to write to the file.");
}

fn read_file(file: &str) {
    let mut fs_handle = std::fs::OpenOptions::new()
        .read(true)
        .open(file)
        .expect("Failed to read the file.");

    let mut content = String::new();
    fs_handle
        .read_to_string(&mut content)
        .expect("Failed to read the file.");

    let mut countdown: u16 = 3;

    cls();
    println!(">> {}", content);

    while 0 < countdown {
        // \r moves the cursor to column 0 of the current line; \x1b[K erases from the cursor to end of line. So you overwrite just that line. \n moves the cursor to the next line so can't use it.
        print!("\rBack to menu in {countdown} seconds...\x1b[K");
        let _ = io::stdout().flush();
        std::thread::sleep(std::time::Duration::from_secs(1));
        countdown -= 1;
    }

    cls();
}

fn update_file(file: &str) {
    let editor: String = std::env::var("EDITOR")
        .or_else(|_| std::env::var("VISUAL"))
        .unwrap_or_else(|_| "vi".to_string());

    let status = Command::new(&editor)
        .arg(file)
        .status()
        .expect("Failed to run the editor program!");

    if !status.success() {
        eprintln!("{editor} cannot be opened with {status}")
    }
}

fn delete_file(file: &str) {
    std::fs::remove_file(file).expect("Failed to delete the file.");
}

// ========= Terminal Config =========>

fn cbreak_on() -> Result<libc::termios> {
    let fd = io::stdin().as_raw_fd();

    let mut orig = MaybeUninit::<libc::termios>::uninit();

    if unsafe { libc::tcgetattr(fd, orig.as_mut_ptr()) } != 0 {
        return Err(io::Error::last_os_error());
    }
    let orig = unsafe { orig.assume_init() };

    let mut raw = orig;

    raw.c_lflag &= !(libc::ICANON | libc::ECHO);
    raw.c_cc[libc::VMIN] = 1;
    raw.c_cc[libc::VTIME] = 0;

    if unsafe { libc::tcsetattr(fd, libc::TCSANOW, &raw) } != 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(orig)
}

fn cbreak_off(orig: &libc::termios) -> io::Result<()> {
    let fd = io::stdin().as_raw_fd();
    if unsafe { libc::tcsetattr(fd, libc::TCSANOW, orig) } != 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}
