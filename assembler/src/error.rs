use std::borrow::Cow;

#[derive(Clone, Debug)]
pub struct Position {
    pub cursor: usize,
    pub line: usize,
    pub character: usize,
}

#[derive(Debug)]
pub struct Error<'a> {
    pub message: Cow<'a, str>,
    pub from: Position,
    pub to: Position,
}

pub type Result<'a, T> = std::result::Result<T, Error<'a>>;
