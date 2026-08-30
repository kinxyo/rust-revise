// ========= Helper Fn(s) =========>

use std::io::{self, Write};

pub fn cls() {
    print!("\x1b[2J\x1b[H");
}

pub fn prompt(msg: &str) -> String {
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
