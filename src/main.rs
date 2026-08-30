// INFO:
// CLI tool for basic file management.
// Learning/Revising project; Didn't bother with error handling much.

mod utils;

mod termios;

mod files;

mod menu;
use menu::Menu;

use std::io::{self, Read, Result};

use crate::termios::{cbreak_off, cbreak_on};

// ========= Main Fn(s) =========>

fn main() -> Result<()> {
    crate::utils::cls();

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

        Menu::exec_user_option(buf[0]);
    }
    Ok(())
}
