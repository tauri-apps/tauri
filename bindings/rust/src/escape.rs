use std::fmt::{self, Write};

/// Escape a string to pass it into JavaScript.
///
/// # Example
///
/// ```rust,no_run
/// # use web_view::WebView;
/// # use std::mem;
/// #
/// # let mut view: WebView<()> = unsafe { mem::uninitialized() };
/// #
/// let string = "Hello, world!";
///
/// // Calls the function callback with "Hello, world!" as its parameter.
///
/// view.eval(&format!("callback({});", web_view::escape(string)));
/// ```
pub fn escape(string: &str) -> Escaper<'_> {
  Escaper(string)
}

// "All code points may appear literally in a string literal except for the
// closing quote code points, U+005C (REVERSE SOLIDUS), U+000D (CARRIAGE
// RETURN), U+2028 (LINE SEPARATOR), U+2029 (PARAGRAPH SEPARATOR), and U+000A
// (LINE FEED)." - ES6 Specification

pub struct Escaper<'a>(&'a str);

const SPECIAL: &[char] = &[
  '\n',       // U+000A (LINE FEED)
  '\r',       // U+000D (CARRIAGE RETURN)
  '\'',       // U+0027 (APOSTROPHE)
  '\\',       // U+005C (REVERSE SOLIDUS)
  '\u{2028}', // U+2028 (LINE SEPARATOR)
  '\u{2029}', // U+2029 (PARAGRAPH SEPARATOR)
];

impl<'a> fmt::Display for Escaper<'a> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let &Escaper(mut string) = self;

    f.write_char('\'')?;

    while !string.is_empty() {
      if let Some(i) = string.find(SPECIAL) {
        if i > 0 {
          f.write_str(&string[..i])?;
        }

        let mut chars = string[i..].chars();

        f.write_str(match chars.next().unwrap() {
          '\n' => "\\n",
          '\r' => "\\r",
          '\'' => "\\'",
          '\\' => "\\\\",
          '\u{2028}' => "\\u2028",
          '\u{2029}' => "\\u2029",
          _ => unreachable!(),
        })?;

        string = chars.as_str();
      } else {
        f.write_str(string)?;
        break;
      }
    }

    f.write_char('\'')?;

    Ok(())
  }
}

#[test]
fn test() {
  let plain = "ABC \n\r' abc \\  \u{2028}   \u{2029}123";
  let escaped = escape(plain).to_string();
  assert!(escaped == "'ABC \\n\\r\\' abc \\\\  \\u2028   \\u2029123'");
}
