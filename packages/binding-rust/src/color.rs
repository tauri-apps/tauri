/// An RGBA color.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl From<(u8, u8, u8, u8)> for Color {
    fn from(tuple: (u8, u8, u8, u8)) -> Color {
        Color {
            r: tuple.0,
            g: tuple.1,
            b: tuple.2,
            a: tuple.3,
        }
    }
}

impl From<(u8, u8, u8)> for Color {
    fn from(tuple: (u8, u8, u8)) -> Color {
        Color {
            r: tuple.0,
            g: tuple.1,
            b: tuple.2,
            a: 255,
        }
    }
}

impl From<[u8; 4]> for Color {
    fn from(array: [u8; 4]) -> Color {
        Color {
            r: array[0],
            g: array[1],
            b: array[2],
            a: array[3],
        }
    }
}

impl From<[u8; 3]> for Color {
    fn from(array: [u8; 3]) -> Color {
        Color {
            r: array[0],
            g: array[1],
            b: array[2],
            a: 255,
        }
    }
}
