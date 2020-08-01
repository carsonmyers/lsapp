use std::cmp::{Ord, Ordering};
use std::ops::{Add, AddAssign, Sub, SubAssign};

#[derive(PartialEq, Eq, PartialOrd, Clone, Copy, Debug)]
pub struct Position {
    pub row: u32,
    pub col: u32,
    pub idx: u32,
}

impl Ord for Position {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.row.cmp(&other.row) {
            Ordering::Equal => self.col.cmp(&other.col),
            c @ _ => c,
        }
    }
}

impl Add<u32> for Position {
    type Output = Self;

    fn add(self, other: u32) -> Self::Output {
        Position { row: self.row, col: self.col + other, idx: self.idx + other }
    }
}

impl AddAssign<u32> for Position {
    fn add_assign(&mut self, other: u32) {
        self.col += other;
        self.idx += other
    }
}

impl Sub<u32> for Position {
    type Output = Self;

    fn sub(self, other: u32) -> Self::Output {
        Position { row: self.row, col: self.col - other, idx: self.idx - other }
    }
}

impl SubAssign<u32> for Position {
    fn sub_assign(&mut self, other: u32) {
        self.col -= other;
        self.idx -= other;
    }
}

impl Position {
    pub fn new() -> Position {
        Position { row: 0, col: 0, idx: 0 }
    }

    pub fn newline(&mut self) {
        self.row += 1;
        self.col = 0;
    }
}

#[derive(PartialEq, Eq, PartialOrd, Clone, Copy)]
pub struct Span {
    pub start: Position,
    pub end: Position,
}

impl Ord for Span {
    fn cmp(&self, other: &Self) -> Ordering {
        self.cmp(&other)
    }
}

impl Add<u32> for Span {
    type Output = Self;

    fn add(self, other: u32) -> Self::Output {
        Span {
            start: self.start,
            end: self.end + other,
        }
    }
}

impl AddAssign<u32> for Span {
    fn add_assign(&mut self, other: u32) {
        self.end += other
    }
}

impl Sub<u32> for Span {
    type Output = Self;

    fn sub(self, other: u32) -> Self::Output {
        let res = Span {
            start: self.start,
            end: self.end - other,
        };

        if res.start > res.end {
            panic!("Invalid span: start and end are inverted");
        }

        res
    }
}

impl SubAssign<u32> for Span {
    fn sub_assign(&mut self, other: u32) {
        self.end -= other;

        if self.start > self.end {
            panic!("Invalid span: start and end are inverted");
        }
    }
}

impl Span {
    pub fn new(start: Position, end: Position) -> Span {
        Span { start, end }
    }

    pub fn start(start: Position) -> Span {
        Span { start, end: start }
    }

    pub fn finish(&mut self, end: Position) {
        if self.start > end {
            panic!("Invalid span: start and end are inverted");
        }

        self.end = end;
    }
}
