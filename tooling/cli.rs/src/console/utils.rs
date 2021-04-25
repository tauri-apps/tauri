// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::borrow::Cow;
use std::collections::BTreeSet;
use std::env;
use std::fmt;
use std::sync::atomic::{AtomicBool, Ordering};

use super::term::{wants_emoji, Term};
use lazy_static::lazy_static;

use super::ansi::{strip_ansi_codes, AnsiCodeIterator};

fn default_colors_enabled(out: &Term) -> bool {
  (out.features().colors_supported() && &env::var("CLICOLOR").unwrap_or_else(|_| "1".into()) != "0")
    || &env::var("CLICOLOR_FORCE").unwrap_or_else(|_| "0".into()) != "0"
}

lazy_static! {
  static ref STDOUT_COLORS: AtomicBool = AtomicBool::new(default_colors_enabled(&Term::stdout()));
  static ref STDERR_COLORS: AtomicBool = AtomicBool::new(default_colors_enabled(&Term::stderr()));
}

/// Returns `true` if colors should be enabled for stdout.
///
/// This honors the [clicolors spec](http://bixense.com/clicolors/).
///
/// * `CLICOLOR != 0`: ANSI colors are supported and should be used when the program isn't piped.
/// * `CLICOLOR == 0`: Don't output ANSI color escape codes.
/// * `CLICOLOR_FORCE != 0`: ANSI colors should be enabled no matter what.
#[inline]
pub fn colors_enabled() -> bool {
  STDOUT_COLORS.load(Ordering::Relaxed)
}

/// Forces colorization on or off for stdout.
///
/// This overrides the default for the current process and changes the return value of the
/// `colors_enabled` function.
#[inline]
pub fn set_colors_enabled(val: bool) {
  STDOUT_COLORS.store(val, Ordering::Relaxed)
}

/// Returns `true` if colors should be enabled for stderr.
///
/// This honors the [clicolors spec](http://bixense.com/clicolors/).
///
/// * `CLICOLOR != 0`: ANSI colors are supported and should be used when the program isn't piped.
/// * `CLICOLOR == 0`: Don't output ANSI color escape codes.
/// * `CLICOLOR_FORCE != 0`: ANSI colors should be enabled no matter what.
#[inline]
pub fn colors_enabled_stderr() -> bool {
  STDERR_COLORS.load(Ordering::Relaxed)
}

/// Forces colorization on or off for stderr.
///
/// This overrides the default for the current process and changes the return value of the
/// `colors_enabled` function.
#[inline]
pub fn set_colors_enabled_stderr(val: bool) {
  STDERR_COLORS.store(val, Ordering::Relaxed)
}

/// Measure the width of a string in terminal characters.
pub fn measure_text_width(s: &str) -> usize {
  str_width(&strip_ansi_codes(s))
}

/// A terminal color.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Color {
  Black,
  Red,
  Green,
  Yellow,
  Blue,
  Magenta,
  Cyan,
  White,
  Color256(u8),
}

impl Color {
  #[inline]
  fn ansi_num(self) -> usize {
    match self {
      Color::Black => 0,
      Color::Red => 1,
      Color::Green => 2,
      Color::Yellow => 3,
      Color::Blue => 4,
      Color::Magenta => 5,
      Color::Cyan => 6,
      Color::White => 7,
      Color::Color256(x) => x as usize,
    }
  }

  #[inline]
  fn is_color256(self) -> bool {
    #[allow(clippy::match_like_matches_macro)]
    match self {
      Color::Color256(_) => true,
      _ => false,
    }
  }
}

/// A terminal style attribute.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Ord, PartialOrd)]
pub enum Attribute {
  Bold,
  Dim,
  Italic,
  Underlined,
  Blink,
  Reverse,
  Hidden,
}

impl Attribute {
  #[inline]
  fn ansi_num(self) -> usize {
    match self {
      Attribute::Bold => 1,
      Attribute::Dim => 2,
      Attribute::Italic => 3,
      Attribute::Underlined => 4,
      Attribute::Blink => 5,
      Attribute::Reverse => 7,
      Attribute::Hidden => 8,
    }
  }
}

/// Defines the alignment for padding operations.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Alignment {
  Left,
  Center,
  Right,
}

/// A stored style that can be applied.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Style {
  fg: Option<Color>,
  bg: Option<Color>,
  fg_bright: bool,
  bg_bright: bool,
  attrs: BTreeSet<Attribute>,
  force: Option<bool>,
  for_stderr: bool,
}

impl Default for Style {
  fn default() -> Style {
    Style::new()
  }
}

impl Style {
  /// Returns an empty default style.
  pub fn new() -> Style {
    Style {
      fg: None,
      bg: None,
      fg_bright: false,
      bg_bright: false,
      attrs: BTreeSet::new(),
      force: None,
      for_stderr: false,
    }
  }

  /// Creates a style from a dotted string.
  ///
  /// Effectively the string is split at each dot and then the
  /// terms in between are applied.  For instance `red.on_blue` will
  /// create a string that is red on blue background.  Unknown terms
  /// are ignored.
  pub fn from_dotted_str(s: &str) -> Style {
    let mut rv = Style::new();
    for part in s.split('.') {
      rv = match part {
        "black" => rv.black(),
        "red" => rv.red(),
        "green" => rv.green(),
        "yellow" => rv.yellow(),
        "blue" => rv.blue(),
        "magenta" => rv.magenta(),
        "cyan" => rv.cyan(),
        "white" => rv.white(),
        "bright" => rv.bright(),
        "on_black" => rv.on_black(),
        "on_red" => rv.on_red(),
        "on_green" => rv.on_green(),
        "on_yellow" => rv.on_yellow(),
        "on_blue" => rv.on_blue(),
        "on_magenta" => rv.on_magenta(),
        "on_cyan" => rv.on_cyan(),
        "on_white" => rv.on_white(),
        "on_bright" => rv.on_bright(),
        "bold" => rv.bold(),
        "dim" => rv.dim(),
        "underlined" => rv.underlined(),
        "blink" => rv.blink(),
        "reverse" => rv.reverse(),
        "hidden" => rv.hidden(),
        _ => {
          continue;
        }
      };
    }
    rv
  }

  /// Apply the style to something that can be displayed.
  pub fn apply_to<D>(&self, val: D) -> StyledObject<D> {
    StyledObject {
      style: self.clone(),
      val,
    }
  }

  /// Forces styling on or off.
  ///
  /// This overrides the detection from `clicolors-control`.
  #[inline]
  pub fn force_styling(mut self, value: bool) -> Style {
    self.force = Some(value);
    self
  }

  /// Specifies that style is applying to something being written on stderr.
  #[inline]
  pub fn for_stderr(mut self) -> Style {
    self.for_stderr = true;
    self
  }

  /// Specifies that style is applying to something being written on stdout.
  ///
  /// This is the default behaviour.
  #[inline]
  pub fn for_stdout(mut self) -> Style {
    self.for_stderr = false;
    self
  }

  /// Sets a foreground color.
  #[inline]
  pub fn fg(mut self, color: Color) -> Style {
    self.fg = Some(color);
    self
  }

  /// Sets a background color.
  #[inline]
  pub fn bg(mut self, color: Color) -> Style {
    self.bg = Some(color);
    self
  }

  /// Adds a attr.
  #[inline]
  pub fn attr(mut self, attr: Attribute) -> Style {
    self.attrs.insert(attr);
    self
  }

  #[inline]
  pub fn black(self) -> Style {
    self.fg(Color::Black)
  }
  #[inline]
  pub fn red(self) -> Style {
    self.fg(Color::Red)
  }
  #[inline]
  pub fn green(self) -> Style {
    self.fg(Color::Green)
  }
  #[inline]
  pub fn yellow(self) -> Style {
    self.fg(Color::Yellow)
  }
  #[inline]
  pub fn blue(self) -> Style {
    self.fg(Color::Blue)
  }
  #[inline]
  pub fn magenta(self) -> Style {
    self.fg(Color::Magenta)
  }
  #[inline]
  pub fn cyan(self) -> Style {
    self.fg(Color::Cyan)
  }
  #[inline]
  pub fn white(self) -> Style {
    self.fg(Color::White)
  }
  #[inline]
  pub fn color256(self, color: u8) -> Style {
    self.fg(Color::Color256(color))
  }

  #[inline]
  pub fn bright(mut self) -> Style {
    self.fg_bright = true;
    self
  }

  #[inline]
  pub fn on_black(self) -> Style {
    self.bg(Color::Black)
  }
  #[inline]
  pub fn on_red(self) -> Style {
    self.bg(Color::Red)
  }
  #[inline]
  pub fn on_green(self) -> Style {
    self.bg(Color::Green)
  }
  #[inline]
  pub fn on_yellow(self) -> Style {
    self.bg(Color::Yellow)
  }
  #[inline]
  pub fn on_blue(self) -> Style {
    self.bg(Color::Blue)
  }
  #[inline]
  pub fn on_magenta(self) -> Style {
    self.bg(Color::Magenta)
  }
  #[inline]
  pub fn on_cyan(self) -> Style {
    self.bg(Color::Cyan)
  }
  #[inline]
  pub fn on_white(self) -> Style {
    self.bg(Color::White)
  }
  #[inline]
  pub fn on_color256(self, color: u8) -> Style {
    self.bg(Color::Color256(color))
  }

  #[inline]
  pub fn on_bright(mut self) -> Style {
    self.bg_bright = true;
    self
  }

  #[inline]
  pub fn bold(self) -> Style {
    self.attr(Attribute::Bold)
  }
  #[inline]
  pub fn dim(self) -> Style {
    self.attr(Attribute::Dim)
  }
  #[inline]
  pub fn italic(self) -> Style {
    self.attr(Attribute::Italic)
  }
  #[inline]
  pub fn underlined(self) -> Style {
    self.attr(Attribute::Underlined)
  }
  #[inline]
  pub fn blink(self) -> Style {
    self.attr(Attribute::Blink)
  }
  #[inline]
  pub fn reverse(self) -> Style {
    self.attr(Attribute::Reverse)
  }
  #[inline]
  pub fn hidden(self) -> Style {
    self.attr(Attribute::Hidden)
  }
}

/// Wraps an object for formatting for styling.
///
/// Example:
///
/// ```rust,no_run
/// # use console::style;
/// format!("Hello {}", style("World").cyan());
/// ```
///
/// This is a shortcut for making a new style and applying it
/// to a value:
///
/// ```rust,no_run
/// # use console::Style;
/// format!("Hello {}", Style::new().cyan().apply_to("World"));
/// ```
pub fn style<D>(val: D) -> StyledObject<D> {
  Style::new().apply_to(val)
}

/// A formatting wrapper that can be styled for a terminal.
#[derive(Clone)]
pub struct StyledObject<D> {
  style: Style,
  val: D,
}

impl<D> StyledObject<D> {
  /// Forces styling on or off.
  ///
  /// This overrides the detection from `clicolors-control`.
  #[inline]
  pub fn force_styling(mut self, value: bool) -> StyledObject<D> {
    self.style = self.style.force_styling(value);
    self
  }

  /// Specifies that style is applying to something being written on stderr
  #[inline]
  pub fn for_stderr(mut self) -> StyledObject<D> {
    self.style = self.style.for_stderr();
    self
  }

  /// Specifies that style is applying to something being written on stdout
  ///
  /// This is the default
  #[inline]
  pub fn for_stdout(mut self) -> StyledObject<D> {
    self.style = self.style.for_stdout();
    self
  }

  /// Sets a foreground color.
  #[inline]
  pub fn fg(mut self, color: Color) -> StyledObject<D> {
    self.style = self.style.fg(color);
    self
  }

  /// Sets a background color.
  #[inline]
  pub fn bg(mut self, color: Color) -> StyledObject<D> {
    self.style = self.style.bg(color);
    self
  }

  /// Adds a attr.
  #[inline]
  pub fn attr(mut self, attr: Attribute) -> StyledObject<D> {
    self.style = self.style.attr(attr);
    self
  }

  #[inline]
  pub fn black(self) -> StyledObject<D> {
    self.fg(Color::Black)
  }
  #[inline]
  pub fn red(self) -> StyledObject<D> {
    self.fg(Color::Red)
  }
  #[inline]
  pub fn green(self) -> StyledObject<D> {
    self.fg(Color::Green)
  }
  #[inline]
  pub fn yellow(self) -> StyledObject<D> {
    self.fg(Color::Yellow)
  }
  #[inline]
  pub fn blue(self) -> StyledObject<D> {
    self.fg(Color::Blue)
  }
  #[inline]
  pub fn magenta(self) -> StyledObject<D> {
    self.fg(Color::Magenta)
  }
  #[inline]
  pub fn cyan(self) -> StyledObject<D> {
    self.fg(Color::Cyan)
  }
  #[inline]
  pub fn white(self) -> StyledObject<D> {
    self.fg(Color::White)
  }
  #[inline]
  pub fn color256(self, color: u8) -> StyledObject<D> {
    self.fg(Color::Color256(color))
  }

  #[inline]
  pub fn bright(mut self) -> StyledObject<D> {
    self.style = self.style.bright();
    self
  }

  #[inline]
  pub fn on_black(self) -> StyledObject<D> {
    self.bg(Color::Black)
  }
  #[inline]
  pub fn on_red(self) -> StyledObject<D> {
    self.bg(Color::Red)
  }
  #[inline]
  pub fn on_green(self) -> StyledObject<D> {
    self.bg(Color::Green)
  }
  #[inline]
  pub fn on_yellow(self) -> StyledObject<D> {
    self.bg(Color::Yellow)
  }
  #[inline]
  pub fn on_blue(self) -> StyledObject<D> {
    self.bg(Color::Blue)
  }
  #[inline]
  pub fn on_magenta(self) -> StyledObject<D> {
    self.bg(Color::Magenta)
  }
  #[inline]
  pub fn on_cyan(self) -> StyledObject<D> {
    self.bg(Color::Cyan)
  }
  #[inline]
  pub fn on_white(self) -> StyledObject<D> {
    self.bg(Color::White)
  }
  #[inline]
  pub fn on_color256(self, color: u8) -> StyledObject<D> {
    self.bg(Color::Color256(color))
  }

  #[inline]
  pub fn on_bright(mut self) -> StyledObject<D> {
    self.style = self.style.on_bright();
    self
  }

  #[inline]
  pub fn bold(self) -> StyledObject<D> {
    self.attr(Attribute::Bold)
  }
  #[inline]
  pub fn dim(self) -> StyledObject<D> {
    self.attr(Attribute::Dim)
  }
  #[inline]
  pub fn italic(self) -> StyledObject<D> {
    self.attr(Attribute::Italic)
  }
  #[inline]
  pub fn underlined(self) -> StyledObject<D> {
    self.attr(Attribute::Underlined)
  }
  #[inline]
  pub fn blink(self) -> StyledObject<D> {
    self.attr(Attribute::Blink)
  }
  #[inline]
  pub fn reverse(self) -> StyledObject<D> {
    self.attr(Attribute::Reverse)
  }
  #[inline]
  pub fn hidden(self) -> StyledObject<D> {
    self.attr(Attribute::Hidden)
  }
}

macro_rules! impl_fmt {
  ($name:ident) => {
    impl<D: fmt::$name> fmt::$name for StyledObject<D> {
      fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut reset = false;
        if self
          .style
          .force
          .unwrap_or_else(|| match self.style.for_stderr {
            true => colors_enabled_stderr(),
            false => colors_enabled(),
          })
        {
          if let Some(fg) = self.style.fg {
            if fg.is_color256() {
              write!(f, "\x1b[38;5;{}m", fg.ansi_num())?;
            } else if self.style.fg_bright {
              write!(f, "\x1b[38;5;{}m", fg.ansi_num() + 8)?;
            } else {
              write!(f, "\x1b[{}m", fg.ansi_num() + 30)?;
            }
            reset = true;
          }
          if let Some(bg) = self.style.bg {
            if bg.is_color256() {
              write!(f, "\x1b[48;5;{}m", bg.ansi_num())?;
            } else if self.style.bg_bright {
              write!(f, "\x1b[48;5;{}m", bg.ansi_num() + 8)?;
            } else {
              write!(f, "\x1b[{}m", bg.ansi_num() + 40)?;
            }
            reset = true;
          }
          for attr in &self.style.attrs {
            write!(f, "\x1b[{}m", attr.ansi_num())?;
            reset = true;
          }
        }
        fmt::$name::fmt(&self.val, f)?;
        if reset {
          write!(f, "\x1b[0m")?;
        }
        Ok(())
      }
    }
  };
}

impl_fmt!(Binary);
impl_fmt!(Debug);
impl_fmt!(Display);
impl_fmt!(LowerExp);
impl_fmt!(LowerHex);
impl_fmt!(Octal);
impl_fmt!(Pointer);
impl_fmt!(UpperExp);
impl_fmt!(UpperHex);

/// "Intelligent" emoji formatter.
///
/// This struct intelligently wraps an emoji so that it is rendered
/// only on systems that want emojis and renders a fallback on others.
///
/// Example:
///
/// ```rust
/// use console::Emoji;
/// println!("[3/4] {}Downloading ...", Emoji("🚚 ", ""));
/// println!("[4/4] {} Done!", Emoji("✨", ":-)"));
/// ```
#[derive(Copy, Clone)]
pub struct Emoji<'a, 'b>(pub &'a str, pub &'b str);

impl<'a, 'b> Emoji<'a, 'b> {
  pub fn new(emoji: &'a str, fallback: &'b str) -> Emoji<'a, 'b> {
    Emoji(emoji, fallback)
  }
}

impl<'a, 'b> fmt::Display for Emoji<'a, 'b> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    if wants_emoji() {
      write!(f, "{}", self.0)
    } else {
      write!(f, "{}", self.1)
    }
  }
}

fn str_width(s: &str) -> usize {
  #[cfg(feature = "unicode-width")]
  {
    use unicode_width::UnicodeWidthStr;
    s.width()
  }
  #[cfg(not(feature = "unicode-width"))]
  {
    s.chars().count()
  }
}

fn char_width(c: char) -> usize {
  #[cfg(feature = "unicode-width")]
  {
    use unicode_width::UnicodeWidthChar;
    c.width().unwrap_or(0)
  }
  #[cfg(not(feature = "unicode-width"))]
  {
    let _c = c;
    1
  }
}

/// Truncates a string to a certain number of characters.
///
/// This ensures that escape codes are not screwed up in the process.
/// If the maximum length is hit the string will be truncated but
/// escapes code will still be honored.  If truncation takes place
/// the tail string will be appended.
pub fn truncate_str<'a>(s: &'a str, width: usize, tail: &str) -> Cow<'a, str> {
  {
    use std::cmp::Ordering;
    let mut iter = AnsiCodeIterator::new(s);
    let mut length = 0;
    let mut rv = None;

    while let Some(item) = iter.next() {
      match item {
        (s, false) => {
          if rv.is_none() {
            if str_width(s) + length > width - str_width(tail) {
              let ts = iter.current_slice();

              let mut s_byte = 0;
              let mut s_width = 0;
              let rest_width = width - str_width(tail) - length;
              for c in s.chars() {
                s_byte += c.len_utf8();
                s_width += char_width(c);
                match s_width.cmp(&rest_width) {
                  Ordering::Equal => break,
                  Ordering::Greater => {
                    s_byte -= c.len_utf8();
                    break;
                  }
                  Ordering::Less => continue,
                }
              }

              let idx = ts.len() - s.len() + s_byte;
              let mut buf = ts[..idx].to_string();
              buf.push_str(tail);
              rv = Some(buf);
            }
            length += str_width(s);
          }
        }
        (s, true) => {
          if rv.is_some() {
            rv.as_mut().unwrap().push_str(s);
          }
        }
      }
    }

    if let Some(buf) = rv {
      Cow::Owned(buf)
    } else {
      Cow::Borrowed(s)
    }
  }
}

/// Pads a string to fill a certain number of characters.
///
/// This will honor ansi codes correctly and allows you to align a string
/// on the left, right or centered.  Additionally truncation can be enabled
/// by setting `truncate` to a string that should be used as a truncation
/// marker.
pub fn pad_str<'a>(
  s: &'a str,
  width: usize,
  align: Alignment,
  truncate: Option<&str>,
) -> Cow<'a, str> {
  pad_str_with(s, width, align, truncate, ' ')
}
/// Pads a string with specific padding to fill a certain number of characters.
///
/// This will honor ansi codes correctly and allows you to align a string
/// on the left, right or centered.  Additionally truncation can be enabled
/// by setting `truncate` to a string that should be used as a truncation
/// marker.
pub fn pad_str_with<'a>(
  s: &'a str,
  width: usize,
  align: Alignment,
  truncate: Option<&str>,
  pad: char,
) -> Cow<'a, str> {
  let cols = measure_text_width(s);

  if cols >= width {
    return match truncate {
      None => Cow::Borrowed(s),
      Some(tail) => truncate_str(s, width, tail),
    };
  }

  let diff = width - cols;

  let (left_pad, right_pad) = match align {
    Alignment::Left => (0, diff),
    Alignment::Right => (diff, 0),
    Alignment::Center => (diff / 2, diff - diff / 2),
  };

  let mut rv = String::new();
  for _ in 0..left_pad {
    rv.push(pad);
  }
  rv.push_str(s);
  for _ in 0..right_pad {
    rv.push(pad);
  }
  Cow::Owned(rv)
}

#[test]
fn test_text_width() {
  let s = style("foo")
    .red()
    .on_black()
    .bold()
    .force_styling(true)
    .to_string();
  assert_eq!(measure_text_width(&s), 3);
}

#[test]
#[cfg(all(feature = "unicode-width", feature = "ansi-parsing"))]
fn test_truncate_str() {
  let s = format!("foo {}", style("bar").red().force_styling(true));
  assert_eq!(
    &truncate_str(&s, 5, ""),
    &format!("foo {}", style("b").red().force_styling(true))
  );
  let s = format!("foo {}", style("bar").red().force_styling(true));
  assert_eq!(
    &truncate_str(&s, 5, "!"),
    &format!("foo {}", style("!").red().force_styling(true))
  );
  let s = format!("foo {} baz", style("bar").red().force_styling(true));
  assert_eq!(
    &truncate_str(&s, 10, "..."),
    &format!("foo {}...", style("bar").red().force_styling(true))
  );
  let s = format!("foo {}", style("バー").red().force_styling(true));
  assert_eq!(
    &truncate_str(&s, 5, ""),
    &format!("foo {}", style("").red().force_styling(true))
  );
  let s = format!("foo {}", style("バー").red().force_styling(true));
  assert_eq!(
    &truncate_str(&s, 6, ""),
    &format!("foo {}", style("バ").red().force_styling(true))
  );
}

#[test]
fn test_truncate_str_no_ansi() {
  assert_eq!(&truncate_str("foo bar", 5, ""), "foo b");
  assert_eq!(&truncate_str("foo bar", 5, "!"), "foo !");
  assert_eq!(&truncate_str("foo bar baz", 10, "..."), "foo bar...");
}

#[test]
fn test_pad_str() {
  assert_eq!(pad_str("foo", 7, Alignment::Center, None), "  foo  ");
  assert_eq!(pad_str("foo", 7, Alignment::Left, None), "foo    ");
  assert_eq!(pad_str("foo", 7, Alignment::Right, None), "    foo");
  assert_eq!(pad_str("foo", 3, Alignment::Left, None), "foo");
  assert_eq!(pad_str("foobar", 3, Alignment::Left, None), "foobar");
  assert_eq!(pad_str("foobar", 3, Alignment::Left, Some("")), "foo");
  assert_eq!(
    pad_str("foobarbaz", 6, Alignment::Left, Some("...")),
    "foo..."
  );
}

#[test]
fn test_pad_str_with() {
  assert_eq!(
    pad_str_with("foo", 7, Alignment::Center, None, '#'),
    "##foo##"
  );
  assert_eq!(
    pad_str_with("foo", 7, Alignment::Left, None, '#'),
    "foo####"
  );
  assert_eq!(
    pad_str_with("foo", 7, Alignment::Right, None, '#'),
    "####foo"
  );
  assert_eq!(pad_str_with("foo", 3, Alignment::Left, None, '#'), "foo");
  assert_eq!(
    pad_str_with("foobar", 3, Alignment::Left, None, '#'),
    "foobar"
  );
  assert_eq!(
    pad_str_with("foobar", 3, Alignment::Left, Some(""), '#'),
    "foo"
  );
  assert_eq!(
    pad_str_with("foobarbaz", 6, Alignment::Left, Some("..."), '#'),
    "foo..."
  );
}
