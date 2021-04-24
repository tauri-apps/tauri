use std::borrow::Cow;

use regex::{Matches, Regex};

lazy_static::lazy_static! {
    static ref STRIP_ANSI_RE: Regex =
        Regex::new(r"[\x1b\x9b]([()][012AB]|[\[()#;?]*(?:[0-9]{1,4}(?:;[0-9]{0,4})*)?[0-9A-PRZcf-nqry=><])")
            .unwrap();
}

/// Helper function to strip ansi codes.
pub fn strip_ansi_codes(s: &str) -> Cow<str> {
    STRIP_ANSI_RE.replace_all(s, "")
}

/// An iterator over ansi codes in a string.
///
/// This type can be used to scan over ansi codes in a string.
/// It yields tuples in the form `(s, is_ansi)` where `s` is a slice of
/// the original string and `is_ansi` indicates if the slice contains
/// ansi codes or string values.
pub struct AnsiCodeIterator<'a> {
    s: &'a str,
    pending_item: Option<(&'a str, bool)>,
    last_idx: usize,
    cur_idx: usize,
    iter: Matches<'static, 'a>,
}

impl<'a> AnsiCodeIterator<'a> {
    /// Creates a new ansi code iterator.
    pub fn new(s: &'a str) -> AnsiCodeIterator<'a> {
        AnsiCodeIterator {
            s,
            pending_item: None,
            last_idx: 0,
            cur_idx: 0,
            iter: STRIP_ANSI_RE.find_iter(s),
        }
    }

    /// Returns the string slice up to the current match.
    pub fn current_slice(&self) -> &str {
        &self.s[..self.cur_idx]
    }

    /// Returns the string slice from the current match to the end.
    pub fn rest_slice(&self) -> &str {
        &self.s[self.cur_idx..]
    }
}

impl<'a> Iterator for AnsiCodeIterator<'a> {
    type Item = (&'a str, bool);

    fn next(&mut self) -> Option<(&'a str, bool)> {
        if let Some(pending_item) = self.pending_item.take() {
            self.cur_idx += pending_item.0.len();
            Some(pending_item)
        } else if let Some(m) = self.iter.next() {
            let s = &self.s[self.last_idx..m.start()];
            self.last_idx = m.end();
            if s.is_empty() {
                self.cur_idx = m.end();
                Some((m.as_str(), true))
            } else {
                self.cur_idx = m.start();
                self.pending_item = Some((m.as_str(), true));
                Some((s, false))
            }
        } else if self.last_idx < self.s.len() {
            let rv = &self.s[self.last_idx..];
            self.cur_idx = self.s.len();
            self.last_idx = self.s.len();
            Some((rv, false))
        } else {
            None
        }
    }
}

#[test]
fn test_ansi_iter_re_vt100() {
    let s = "\x1b(0lpq\x1b)Benglish";
    let mut iter = AnsiCodeIterator::new(s);
    assert_eq!(iter.next(), Some(("\x1b(0", true)));
    assert_eq!(iter.next(), Some(("lpq", false)));
    assert_eq!(iter.next(), Some(("\x1b)B", true)));
    assert_eq!(iter.next(), Some(("english", false)));
}

#[test]
fn test_ansi_iter_re() {
    use super::style;
    let s = format!("Hello {}!", style("World").red().force_styling(true));
    let mut iter = AnsiCodeIterator::new(&s);
    assert_eq!(iter.next(), Some(("Hello ", false)));
    assert_eq!(iter.current_slice(), "Hello ");
    assert_eq!(iter.rest_slice(), "\x1b[31mWorld\x1b[0m!");
    assert_eq!(iter.next(), Some(("\x1b[31m", true)));
    assert_eq!(iter.current_slice(), "Hello \x1b[31m");
    assert_eq!(iter.rest_slice(), "World\x1b[0m!");
    assert_eq!(iter.next(), Some(("World", false)));
    assert_eq!(iter.current_slice(), "Hello \x1b[31mWorld");
    assert_eq!(iter.rest_slice(), "\x1b[0m!");
    assert_eq!(iter.next(), Some(("\x1b[0m", true)));
    assert_eq!(iter.current_slice(), "Hello \x1b[31mWorld\x1b[0m");
    assert_eq!(iter.rest_slice(), "!");
    assert_eq!(iter.next(), Some(("!", false)));
    assert_eq!(iter.current_slice(), "Hello \x1b[31mWorld\x1b[0m!");
    assert_eq!(iter.rest_slice(), "");
    assert_eq!(iter.next(), None);
}

#[test]
fn test_ansi_iter_re_on_multi() {
    use super::style;
    let s = format!("{}", style("a").red().bold().force_styling(true));
    let mut iter = AnsiCodeIterator::new(&s);
    assert_eq!(iter.next(), Some(("\x1b[31m", true)));
    assert_eq!(iter.current_slice(), "\x1b[31m");
    assert_eq!(iter.rest_slice(), "\x1b[1ma\x1b[0m");
    assert_eq!(iter.next(), Some(("\x1b[1m", true)));
    assert_eq!(iter.current_slice(), "\x1b[31m\x1b[1m");
    assert_eq!(iter.rest_slice(), "a\x1b[0m");
    assert_eq!(iter.next(), Some(("a", false)));
    assert_eq!(iter.current_slice(), "\x1b[31m\x1b[1ma");
    assert_eq!(iter.rest_slice(), "\x1b[0m");
    assert_eq!(iter.next(), Some(("\x1b[0m", true)));
    assert_eq!(iter.current_slice(), "\x1b[31m\x1b[1ma\x1b[0m");
    assert_eq!(iter.rest_slice(), "");
    assert_eq!(iter.next(), None);
}
