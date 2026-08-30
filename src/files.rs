// ========= APIs -- File Management =========>

use crate::utils::{cls, prompt};

use std::{
    io::{self, Read, Write},
    process::Command,
};

pub fn create_file(file: &str) {
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

pub fn read_file(file: &str) {
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

pub fn update_file(file: &str) {
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

pub fn delete_file(file: &str) {
    std::fs::remove_file(file).expect("Failed to delete the file.");
}
