use std::env;
use std::fmt::Display;
use std::fs;
use std::io;
use std::io::{BufRead, BufReader};
use std::os::unix::io::AsRawFd;
use std::str;

use super::kb::Key;
use super::term::Term;

pub use super::common_term::*;

pub const DEFAULT_WIDTH: u16 = 80;

#[inline]
pub fn is_a_terminal(out: &Term) -> bool {
    unsafe { libc::isatty(out.as_raw_fd()) != 0 }
}

pub fn is_a_color_terminal(out: &Term) -> bool {
    if !is_a_terminal(out) {
        return false;
    }

    if env::var("NO_COLOR").is_ok() {
        return false;
    }

    match env::var("TERM") {
        Ok(term) => term != "dumb",
        Err(_) => false,
    }
}

pub fn c_result<F: FnOnce() -> libc::c_int>(f: F) -> io::Result<()> {
    let res = f();
    if res != 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

#[inline]
pub fn terminal_size(out: &Term) -> Option<(u16, u16)> {
    terminal_size::terminal_size_using_fd(out.as_raw_fd()).map(|x| ((x.1).0, (x.0).0))
}

pub fn read_secure() -> io::Result<String> {
    let f_tty;
    let fd = unsafe {
        if libc::isatty(libc::STDIN_FILENO) == 1 {
            f_tty = None;
            libc::STDIN_FILENO
        } else {
            let f = fs::File::open("/dev/tty")?;
            let fd = f.as_raw_fd();
            f_tty = Some(BufReader::new(f));
            fd
        }
    };

    let mut termios = core::mem::MaybeUninit::uninit();
    c_result(|| unsafe { libc::tcgetattr(fd, termios.as_mut_ptr()) })?;
    let mut termios = unsafe { termios.assume_init() };
    let original = termios;
    termios.c_lflag &= !libc::ECHO;
    c_result(|| unsafe { libc::tcsetattr(fd, libc::TCSAFLUSH, &termios) })?;
    let mut rv = String::new();

    let read_rv = if let Some(mut f) = f_tty {
        f.read_line(&mut rv)
    } else {
        io::stdin().read_line(&mut rv)
    };

    c_result(|| unsafe { libc::tcsetattr(fd, libc::TCSAFLUSH, &original) })?;

    read_rv.map(|_| {
        let len = rv.trim_end_matches(&['\r', '\n'][..]).len();
        rv.truncate(len);
        rv
    })
}

fn read_single_char(fd: i32) -> io::Result<Option<char>> {
    let mut pollfd = libc::pollfd {
        fd,
        events: libc::POLLIN,
        revents: 0,
    };

    // timeout of zero means that it will not block
    let ret = unsafe { libc::poll(&mut pollfd as *mut _, 1, 0) };
    if ret < 0 {
        return Err(io::Error::last_os_error());
    }

    let is_ready = pollfd.revents & libc::POLLIN != 0;

    if is_ready {
        // if there is something to be read, take 1 byte from it
        let mut buf: [u8; 1] = [0];

        read_bytes(fd, &mut buf, 1)?;
        Ok(Some(buf[0] as char))
    } else {
        //there is nothing to be read
        Ok(None)
    }
}

// Similar to libc::read. Read count bytes into slice buf from descriptor fd.
// If successful, return the number of bytes read.
// Will return an error if nothing was read, i.e when called at end of file.
fn read_bytes(fd: i32, buf: &mut [u8], count: u8) -> io::Result<u8> {
    let read = unsafe { libc::read(fd, buf.as_mut_ptr() as *mut _, count as usize) };
    if read < 0 {
        Err(io::Error::last_os_error())
    } else if read == 0 {
        Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "Reached end of file",
        ))
    } else if buf[0] == b'\x03' {
        Err(io::Error::new(
            io::ErrorKind::Interrupted,
            "read interrupted",
        ))
    } else {
        Ok(read as u8)
    }
}

pub fn read_single_key() -> io::Result<Key> {
    let tty_f;
    let fd = unsafe {
        if libc::isatty(libc::STDIN_FILENO) == 1 {
            libc::STDIN_FILENO
        } else {
            tty_f = fs::File::open("/dev/tty")?;
            tty_f.as_raw_fd()
        }
    };
    let mut termios = core::mem::MaybeUninit::uninit();
    c_result(|| unsafe { libc::tcgetattr(fd, termios.as_mut_ptr()) })?;
    let mut termios = unsafe { termios.assume_init() };
    let original = termios;
    unsafe { libc::cfmakeraw(&mut termios) };
    c_result(|| unsafe { libc::tcsetattr(fd, libc::TCSADRAIN, &termios) })?;

    let rv = match read_single_char(fd)? {
        Some('\x1b') => {
            // Escape was read, keep reading in case we find a familiar key
            if let Some(c1) = read_single_char(fd)? {
                if c1 == '[' {
                    if let Some(c2) = read_single_char(fd)? {
                        match c2 {
                            'A' => Ok(Key::ArrowUp),
                            'B' => Ok(Key::ArrowDown),
                            'C' => Ok(Key::ArrowRight),
                            'D' => Ok(Key::ArrowLeft),
                            'H' => Ok(Key::Home),
                            'F' => Ok(Key::End),
                            'Z' => Ok(Key::BackTab),
                            _ => {
                                let c3 = read_single_char(fd)?;
                                if let Some(c3) = c3 {
                                    if c3 == '~' {
                                        match c2 {
                                            '1' => Ok(Key::Home), // tmux
                                            '2' => Ok(Key::Insert),
                                            '3' => Ok(Key::Del),
                                            '4' => Ok(Key::End), // tmux
                                            '5' => Ok(Key::PageUp),
                                            '6' => Ok(Key::PageDown),
                                            '7' => Ok(Key::Home), // xrvt
                                            '8' => Ok(Key::End),  // xrvt
                                            _ => Ok(Key::UnknownEscSeq(vec![c1, c2, c3])),
                                        }
                                    } else {
                                        Ok(Key::UnknownEscSeq(vec![c1, c2, c3]))
                                    }
                                } else {
                                    // \x1b[ and 1 more char
                                    Ok(Key::UnknownEscSeq(vec![c1, c2]))
                                }
                            }
                        }
                    } else {
                        // \x1b[ and no more input
                        Ok(Key::UnknownEscSeq(vec![c1]))
                    }
                } else {
                    // char after escape is not [
                    Ok(Key::UnknownEscSeq(vec![c1]))
                }
            } else {
                //nothing after escape
                Ok(Key::Escape)
            }
        }
        Some(c) => {
            let byte = c as u8;
            let mut buf: [u8; 4] = [byte, 0, 0, 0];

            if byte & 224u8 == 192u8 {
                // a two byte unicode character
                read_bytes(fd, &mut buf[1..], 1)?;
                Ok(key_from_utf8(&buf[..2]))
            } else if byte & 240u8 == 224u8 {
                // a three byte unicode character
                read_bytes(fd, &mut buf[1..], 2)?;
                Ok(key_from_utf8(&buf[..3]))
            } else if byte & 248u8 == 240u8 {
                // a four byte unicode character
                read_bytes(fd, &mut buf[1..], 3)?;
                Ok(key_from_utf8(&buf[..4]))
            } else {
                Ok(match c {
                    '\n' | '\r' => Key::Enter,
                    '\x7f' => Key::Backspace,
                    '\t' => Key::Tab,
                    '\x01' => Key::Home,      // Control-A (home)
                    '\x05' => Key::End,       // Control-E (end)
                    '\x08' => Key::Backspace, // Control-H (8) (Identical to '\b')
                    _ => Key::Char(c),
                })
            }
        }
        None => {
            // there is no subsequent byte ready to be read, block and wait for input

            let mut pollfd = libc::pollfd {
                fd,
                events: libc::POLLIN,
                revents: 0,
            };

            // negative timeout means that it will block indefinitely
            let ret = unsafe { libc::poll(&mut pollfd as *mut _, 1, -1) };
            if ret < 0 {
                return Err(io::Error::last_os_error());
            }

            read_single_key()
        }
    };
    c_result(|| unsafe { libc::tcsetattr(fd, libc::TCSADRAIN, &original) })?;

    // if the user hit ^C we want to signal SIGINT to outselves.
    if let Err(ref err) = rv {
        if err.kind() == io::ErrorKind::Interrupted {
            unsafe {
                libc::raise(libc::SIGINT);
            }
        }
    }

    rv
}

pub fn key_from_utf8(buf: &[u8]) -> Key {
    if let Ok(s) = str::from_utf8(buf) {
        if let Some(c) = s.chars().next() {
            return Key::Char(c);
        }
    }
    Key::Unknown
}

#[cfg(not(target_os = "macos"))]
lazy_static::lazy_static! {
    static ref IS_LANG_UTF8: bool = {
        match std::env::var("LANG") {
            Ok(lang) => lang.to_uppercase().ends_with("UTF-8"),
            _ => false,
        }
    };
}

#[cfg(target_os = "macos")]
pub fn wants_emoji() -> bool {
    true
}

#[cfg(not(target_os = "macos"))]
pub fn wants_emoji() -> bool {
    *IS_LANG_UTF8
}

pub fn set_title<T: Display>(title: T) {
    print!("\x1b]0;{}\x07", title);
}
