use std::fmt::{Display, Formatter};

/// Exists to disambiguate tuples holding col/row values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Point {
    pub col: u16,
    pub row: u16,
}

impl Display for Point {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}x{}", self.col, self.row)
    }
}

impl Point {
    /// Creates a Point assuming the tuple is (row, col), not (col, row).
    /// Just use Point::from or (u16, u16).into() for (col, row).
    pub fn flipped(row_col: (u16, u16)) -> Point {
        Point {
            col: row_col.1,
            row: row_col.0,
        }
    }
    

    /// Unpack to a (col: u16, row: u16) pair.
    pub fn unpack(&self) -> (u16, u16) {
        (self.col, self.row)
    }
}

impl From<(u16, u16)> for Point {
    /// Assumes the (u16, u16) tuple is in the form (col, row).
    fn from(value: (u16, u16)) -> Self {
        Point {
            col: value.0,
            row: value.1,
        }
    }
}


impl From<(usize, usize)> for Point {
    /// Assumes the (usize, usize) tuple is in the form (col, row).
    fn from(value: (usize, usize)) -> Self {
        Point {
            col: value.0 as u16,
            row: value.1 as u16,
        }
    }
}

