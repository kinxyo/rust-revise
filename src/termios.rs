// ========= Terminal Config =========>

use std::{
    io::{self, Result},
    mem::MaybeUninit,
    os::fd::AsRawFd,
};

pub fn cbreak_on() -> Result<libc::termios> {
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

pub fn cbreak_off(orig: &libc::termios) -> io::Result<()> {
    let fd = io::stdin().as_raw_fd();
    if unsafe { libc::tcsetattr(fd, libc::TCSANOW, orig) } != 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}
