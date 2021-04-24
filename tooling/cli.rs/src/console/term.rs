use std::fmt::{Debug, Display};
use std::io::{self, Read, Write};
use std::sync::{Arc, Mutex};

#[cfg(unix)]
use std::os::unix::io::{AsRawFd, RawFd};
#[cfg(windows)]
use std::os::windows::io::{AsRawHandle, RawHandle};

use super::{kb::Key, utils::Style};

#[cfg(unix)]
trait TermWrite: Write + Debug + AsRawFd + Send {}
#[cfg(unix)]
impl<T: Write + Debug + AsRawFd + Send> TermWrite for T {}

#[cfg(unix)]
trait TermRead: Read + Debug + AsRawFd + Send {}
#[cfg(unix)]
impl<T: Read + Debug + AsRawFd + Send> TermRead for T {}

#[cfg(unix)]
#[derive(Debug, Clone)]
pub struct ReadWritePair {
    read: Arc<Mutex<dyn TermRead>>,
    write: Arc<Mutex<dyn TermWrite>>,
    style: Style,
}

/// Where the term is writing.
#[derive(Debug, Clone)]
pub enum TermTarget {
    Stdout,
    Stderr,
    #[cfg(unix)]
    ReadWritePair(ReadWritePair),
}

#[derive(Debug)]
pub struct TermInner {
    target: TermTarget,
    buffer: Option<Mutex<Vec<u8>>>,
}

/// The family of the terminal.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TermFamily {
    /// Redirected to a file or file like thing.
    File,
    /// A standard unix terminal.
    UnixTerm,
    /// A cmd.exe like windows console.
    WindowsConsole,
    /// A dummy terminal (for instance on wasm)
    Dummy,
}

/// Gives access to the terminal features.
#[derive(Debug, Clone)]
pub struct TermFeatures<'a>(&'a Term);

impl<'a> TermFeatures<'a> {
    /// Checks if this is a real user attended terminal (`isatty`)
    #[inline]
    pub fn is_attended(&self) -> bool {
        is_a_terminal(self.0)
    }

    /// Checks if colors are supported by this terminal.
    ///
    /// This does not check if colors are enabled.  Currently all terminals
    /// are considered to support colors
    #[inline]
    pub fn colors_supported(&self) -> bool {
        is_a_color_terminal(self.0)
    }

    /// Checks if this terminal is an msys terminal.
    ///
    /// This is sometimes useful to disable features that are known to not
    /// work on msys terminals or require special handling.
    #[inline]
    pub fn is_msys_tty(&self) -> bool {
        #[cfg(windows)]
        {
            msys_tty_on(&self.0)
        }
        #[cfg(not(windows))]
        {
            false
        }
    }

    /// Checks if this terminal wants emojis.
    #[inline]
    pub fn wants_emoji(&self) -> bool {
        self.is_attended() && wants_emoji()
    }

    /// Returns the family of the terminal.
    #[inline]
    pub fn family(&self) -> TermFamily {
        if !self.is_attended() {
            return TermFamily::File;
        }
        #[cfg(windows)]
        {
            TermFamily::WindowsConsole
        }
        #[cfg(unix)]
        {
            TermFamily::UnixTerm
        }
        #[cfg(target_arch = "wasm32")]
        {
            TermFamily::Dummy
        }
    }
}

/// Abstraction around a terminal.
///
/// A terminal can be cloned.  If a buffer is used it's shared across all
/// clones which means it largely acts as a handle.
#[derive(Clone, Debug)]
pub struct Term {
    inner: Arc<TermInner>,
    pub(crate) is_msys_tty: bool,
    pub(crate) is_tty: bool,
}

impl Term {
    fn with_inner(inner: TermInner) -> Term {
        let mut term = Term {
            inner: Arc::new(inner),
            is_msys_tty: false,
            is_tty: false,
        };

        term.is_msys_tty = term.features().is_msys_tty();
        term.is_tty = term.features().is_attended();
        term
    }

    /// Return a new unbuffered terminal
    #[inline]
    pub fn stdout() -> Term {
        Term::with_inner(TermInner {
            target: TermTarget::Stdout,
            buffer: None,
        })
    }

    /// Return a new unbuffered terminal to stderr
    #[inline]
    pub fn stderr() -> Term {
        Term::with_inner(TermInner {
            target: TermTarget::Stderr,
            buffer: None,
        })
    }

    /// Return a new buffered terminal
    pub fn buffered_stdout() -> Term {
        Term::with_inner(TermInner {
            target: TermTarget::Stdout,
            buffer: Some(Mutex::new(vec![])),
        })
    }

    /// Return a new buffered terminal to stderr
    pub fn buffered_stderr() -> Term {
        Term::with_inner(TermInner {
            target: TermTarget::Stderr,
            buffer: Some(Mutex::new(vec![])),
        })
    }

    /// Return a terminal for the given Read/Write pair styled-like Stderr.
    #[cfg(unix)]
    pub fn read_write_pair<R, W>(read: R, write: W) -> Term
    where
        R: Read + Debug + AsRawFd + Send + 'static,
        W: Write + Debug + AsRawFd + Send + 'static,
    {
        Self::read_write_pair_with_style(read, write, Style::new().for_stderr())
    }

    /// Return a terminal for the given Read/Write pair.
    #[cfg(unix)]
    pub fn read_write_pair_with_style<R, W>(read: R, write: W, style: Style) -> Term
    where
        R: Read + Debug + AsRawFd + Send + 'static,
        W: Write + Debug + AsRawFd + Send + 'static,
    {
        Term::with_inner(TermInner {
            target: TermTarget::ReadWritePair(ReadWritePair {
                read: Arc::new(Mutex::new(read)),
                write: Arc::new(Mutex::new(write)),
                style,
            }),
            buffer: None,
        })
    }

    /// Returns the style for the term
    #[inline]
    pub fn style(&self) -> Style {
        match self.inner.target {
            TermTarget::Stderr => Style::new().for_stderr(),
            TermTarget::Stdout => Style::new().for_stdout(),
            #[cfg(unix)]
            TermTarget::ReadWritePair(ReadWritePair { ref style, .. }) => style.clone(),
        }
    }

    /// Returns the target
    #[inline]
    pub fn target(&self) -> TermTarget {
        self.inner.target.clone()
    }

    #[doc(hidden)]
    pub fn write_str(&self, s: &str) -> io::Result<()> {
        match self.inner.buffer {
            Some(ref buffer) => buffer.lock().unwrap().write_all(s.as_bytes()),
            None => self.write_through(s.as_bytes()),
        }
    }

    /// Writes a string to the terminal and adds a newline.
    pub fn write_line(&self, s: &str) -> io::Result<()> {
        match self.inner.buffer {
            Some(ref mutex) => {
                let mut buffer = mutex.lock().unwrap();
                buffer.extend_from_slice(s.as_bytes());
                buffer.push(b'\n');
                Ok(())
            }
            None => self.write_through(format!("{}\n", s).as_bytes()),
        }
    }

    /// Read a single character from the terminal
    ///
    /// This does not echo the character and blocks until a single character
    /// is entered.
    pub fn read_char(&self) -> io::Result<char> {
        loop {
            match self.read_key()? {
                Key::Char(c) => {
                    return Ok(c);
                }
                Key::Enter => {
                    return Ok('\n');
                }
                Key::Unknown => {
                    return Err(io::Error::new(
                        io::ErrorKind::NotConnected,
                        "Not a terminal",
                    ))
                }
                _ => {}
            }
        }
    }

    /// Read a single key form the terminal.
    ///
    /// This does not echo anything.  If the terminal is not user attended
    /// the return value will always be the unknown key.
    pub fn read_key(&self) -> io::Result<Key> {
        if !self.is_tty {
            Ok(Key::Unknown)
        } else {
            read_single_key()
        }
    }

    /// Read one line of input.
    ///
    /// This does not include the trailing newline.  If the terminal is not
    /// user attended the return value will always be an empty string.
    pub fn read_line(&self) -> io::Result<String> {
        if !self.is_tty {
            return Ok("".into());
        }
        let mut rv = String::new();
        io::stdin().read_line(&mut rv)?;
        let len = rv.trim_end_matches(&['\r', '\n'][..]).len();
        rv.truncate(len);
        Ok(rv)
    }

    /// Read one line of input with initial text.
    ///
    /// This does not include the trailing newline.  If the terminal is not
    /// user attended the return value will always be an empty string.
    pub fn read_line_initial_text(&self, initial: &str) -> io::Result<String> {
        if !self.is_tty {
            return Ok("".into());
        }
        self.write_str(initial)?;

        let mut chars: Vec<char> = initial.chars().collect();

        loop {
            match self.read_key()? {
                Key::Backspace => {
                    if chars.pop().is_some() {
                        self.clear_chars(1)?;
                    }
                    self.flush()?;
                }
                Key::Char(chr) => {
                    chars.push(chr);
                    let mut bytes_char = [0; 4];
                    chr.encode_utf8(&mut bytes_char);
                    self.write_str(chr.encode_utf8(&mut bytes_char))?;
                    self.flush()?;
                }
                Key::Enter => break,
                Key::Unknown => {
                    return Err(io::Error::new(
                        io::ErrorKind::NotConnected,
                        "Not a terminal",
                    ))
                }
                _ => (),
            }
        }
        Ok(chars.iter().collect::<String>())
    }

    /// Read securely a line of input.
    ///
    /// This is similar to `read_line` but will not echo the output.  This
    /// also switches the terminal into a different mode where not all
    /// characters might be accepted.
    pub fn read_secure_line(&self) -> io::Result<String> {
        if !self.is_tty {
            return Ok("".into());
        }
        match read_secure() {
            Ok(rv) => {
                self.write_line("")?;
                Ok(rv)
            }
            Err(err) => Err(err),
        }
    }

    /// Flushes internal buffers.
    ///
    /// This forces the contents of the internal buffer to be written to
    /// the terminal.  This is unnecessary for unbuffered terminals which
    /// will automatically flush.
    pub fn flush(&self) -> io::Result<()> {
        if let Some(ref buffer) = self.inner.buffer {
            let mut buffer = buffer.lock().unwrap();
            if !buffer.is_empty() {
                self.write_through(&buffer[..])?;
                buffer.clear();
            }
        }
        Ok(())
    }

    /// Checks if the terminal is indeed a terminal.
    #[inline]
    pub fn is_term(&self) -> bool {
        self.is_tty
    }

    /// Checks for common terminal features.
    #[inline]
    pub fn features(&self) -> TermFeatures<'_> {
        TermFeatures(self)
    }

    /// Returns the terminal size in rows and columns or gets sensible defaults.
    #[inline]
    pub fn size(&self) -> (u16, u16) {
        self.size_checked().unwrap_or((24, DEFAULT_WIDTH))
    }

    /// Returns the terminal size in rows and columns.
    ///
    /// If the size cannot be reliably determined None is returned.
    #[inline]
    pub fn size_checked(&self) -> Option<(u16, u16)> {
        terminal_size(self)
    }

    /// Moves the cursor to `x` and `y`
    #[inline]
    pub fn move_cursor_to(&self, x: usize, y: usize) -> io::Result<()> {
        move_cursor_to(self, x, y)
    }

    /// Moves the cursor up `n` lines
    #[inline]
    pub fn move_cursor_up(&self, n: usize) -> io::Result<()> {
        move_cursor_up(self, n)
    }

    /// Moves the cursor down `n` lines
    #[inline]
    pub fn move_cursor_down(&self, n: usize) -> io::Result<()> {
        move_cursor_down(self, n)
    }

    /// Moves the cursor left `n` lines
    #[inline]
    pub fn move_cursor_left(&self, n: usize) -> io::Result<()> {
        move_cursor_left(self, n)
    }

    /// Moves the cursor down `n` lines
    #[inline]
    pub fn move_cursor_right(&self, n: usize) -> io::Result<()> {
        move_cursor_right(self, n)
    }

    /// Clears the current line.
    ///
    /// The positions the cursor at the beginning of the line again.
    #[inline]
    pub fn clear_line(&self) -> io::Result<()> {
        clear_line(self)
    }

    /// Clear the last `n` lines.
    ///
    /// This positions the cursor at the beginning of the first line
    /// that was cleared.
    pub fn clear_last_lines(&self, n: usize) -> io::Result<()> {
        self.move_cursor_up(n)?;
        for _ in 0..n {
            self.clear_line()?;
            self.move_cursor_down(1)?;
        }
        self.move_cursor_up(n)?;
        Ok(())
    }

    /// Clears the entire screen.
    #[inline]
    pub fn clear_screen(&self) -> io::Result<()> {
        clear_screen(self)
    }

    /// Clears the entire screen.
    #[inline]
    pub fn clear_to_end_of_screen(&self) -> io::Result<()> {
        clear_to_end_of_screen(self)
    }

    /// Clears the last char in the the current line.
    #[inline]
    pub fn clear_chars(&self, n: usize) -> io::Result<()> {
        clear_chars(self, n)
    }

    /// Set the terminal title
    pub fn set_title<T: Display>(&self, title: T) {
        if !self.is_tty {
            return;
        }
        set_title(title);
    }

    /// Makes cursor visible again
    #[inline]
    pub fn show_cursor(&self) -> io::Result<()> {
        show_cursor(self)
    }

    /// Hides cursor
    #[inline]
    pub fn hide_cursor(&self) -> io::Result<()> {
        hide_cursor(self)
    }

    // helpers

    #[cfg(all(windows, feature = "windows-console-colors"))]
    fn write_through(&self, bytes: &[u8]) -> io::Result<()> {
        if self.is_msys_tty || !self.is_tty {
            self.write_through_common(bytes)
        } else {
            use winapi_util::console::Console;

            match self.inner.target {
                TermTarget::Stdout => console_colors(self, Console::stdout()?, bytes),
                TermTarget::Stderr => console_colors(self, Console::stderr()?, bytes),
            }
        }
    }

    #[cfg(not(all(windows, feature = "windows-console-colors")))]
    fn write_through(&self, bytes: &[u8]) -> io::Result<()> {
        self.write_through_common(bytes)
    }

    pub(crate) fn write_through_common(&self, bytes: &[u8]) -> io::Result<()> {
        match self.inner.target {
            TermTarget::Stdout => {
                io::stdout().write_all(bytes)?;
                io::stdout().flush()?;
            }
            TermTarget::Stderr => {
                io::stderr().write_all(bytes)?;
                io::stderr().flush()?;
            }
            #[cfg(unix)]
            TermTarget::ReadWritePair(ReadWritePair { ref write, .. }) => {
                let mut write = write.lock().unwrap();
                write.write_all(bytes)?;
                write.flush()?;
            }
        }
        Ok(())
    }
}

/// A fast way to check if the application has a user attended for stdout.
///
/// This means that stdout is connected to a terminal instead of a
/// file or redirected by other means. This is a shortcut for
/// checking the `is_attended` feature on the stdout terminal.
#[inline]
pub fn user_attended() -> bool {
    Term::stdout().features().is_attended()
}

/// A fast way to check if the application has a user attended for stderr.
///
/// This means that stderr is connected to a terminal instead of a
/// file or redirected by other means. This is a shortcut for
/// checking the `is_attended` feature on the stderr terminal.
#[inline]
pub fn user_attended_stderr() -> bool {
    Term::stderr().features().is_attended()
}

#[cfg(unix)]
impl AsRawFd for Term {
    fn as_raw_fd(&self) -> RawFd {
        match self.inner.target {
            TermTarget::Stdout => libc::STDOUT_FILENO,
            TermTarget::Stderr => libc::STDERR_FILENO,
            TermTarget::ReadWritePair(ReadWritePair { ref write, .. }) => {
                write.lock().unwrap().as_raw_fd()
            }
        }
    }
}

#[cfg(windows)]
impl AsRawHandle for Term {
    fn as_raw_handle(&self) -> RawHandle {
        use winapi::um::processenv::GetStdHandle;
        use winapi::um::winbase::{STD_ERROR_HANDLE, STD_OUTPUT_HANDLE};

        unsafe {
            GetStdHandle(match self.inner.target {
                TermTarget::Stdout => STD_OUTPUT_HANDLE,
                TermTarget::Stderr => STD_ERROR_HANDLE,
            }) as RawHandle
        }
    }
}

impl Write for Term {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self.inner.buffer {
            Some(ref buffer) => buffer.lock().unwrap().write_all(buf),
            None => self.write_through(buf),
        }?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Term::flush(self)
    }
}

impl<'a> Write for &'a Term {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self.inner.buffer {
            Some(ref buffer) => buffer.lock().unwrap().write_all(buf),
            None => self.write_through(buf),
        }?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Term::flush(self)
    }
}

impl Read for Term {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        io::stdin().read(buf)
    }
}

impl<'a> Read for &'a Term {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        io::stdin().read(buf)
    }
}

#[cfg(unix)]
pub use super::unix_term::*;
#[cfg(target_arch = "wasm32")]
pub use super::wasm_term::*;
#[cfg(windows)]
pub use super::windows_term::*;
