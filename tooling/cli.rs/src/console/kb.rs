/// Key mapping
///
/// This is an incomplete mapping of keys that are supported for reading
/// from the keyboard.
#[non_exhaustive]
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Key {
    Unknown,
    /// Unrecognized sequence containing Esc and a list of chars
    UnknownEscSeq(Vec<char>),
    ArrowLeft,
    ArrowRight,
    ArrowUp,
    ArrowDown,
    Enter,
    Escape,
    Backspace,
    Home,
    End,
    Tab,
    BackTab,
    Del,
    Shift,
    Insert,
    PageUp,
    PageDown,
    Char(char),
}
