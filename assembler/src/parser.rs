#![allow(dead_code)]

use crate::instructions::{Instruction, Register, Target};

pub struct Parser<'a> {
    cursor: usize,
    character: usize,
    line: usize,
    inner: &'a [u8],
}

pub struct Position {
    cursor: usize,
    line: usize,
    character: usize,
}

struct Error<'a> {
    message: &'a str,
    from: Position,
    to: Position,
}

impl<'a> Parser<'a> {
    fn register_from_text(value: &[u8]) -> Option<Register> {
        match value {
            b"r0" => Some(Register::GeneralPurpose0),
            b"r1" => Some(Register::GeneralPurpose1),
            b"fl" => Some(Register::Flag),
            b"pc" => Some(Register::ProgramCounter),
            _ => None,
        }
    }
    fn immediate_from_text(value: &[u8]) -> u32 {
        let value = if value.starts_with(b"0x") {
            let value = &value[2..];
            i64::from_str_radix(
                std::str::from_utf8(value).expect("should not be called without utf-8 chars"),
                16,
            )
        } else if value.starts_with(b"0b") {
            let value = &value[2..];
            i64::from_str_radix(
                std::str::from_utf8(value).expect("should not be called without utf-8 chars"),
                2,
            )
        } else {
            println!("'hoh:{value:?}'");
            i64::from_str_radix(
                std::str::from_utf8(value).expect("should not be called without utf-8 chars"),
                10,
            )
        };

        let Ok(value) = value else {
            todo!("invalid number")
        };

        if (value < 0 && (value > i32::MAX as i64 || value < i32::MIN as i64))
            || value > u32::MAX as i64
        {
            todo!("number not within u32 bounds");
        }
        value as u32
    }
    fn parse_target(&mut self) -> Target {
        match self.current() {
            b'[' => {
                self.step();
                self.skip_whitespace();
                let word = self.take_id();
                if let Some(register) = Self::register_from_text(word) {
                    loop {
                        if self.current() == b']' {
                            self.step();
                            break;
                        } else if !self.current().is_ascii_whitespace() || self.done() {
                            todo!("unclosed [")
                        }
                        self.step();
                    }
                    Target::RegisterAddress(register)
                } else {
                    let immediate = Self::immediate_from_text(word);
                    self.skip_whitespace();
                    loop {
                        if self.current() == b']' {
                            self.step();
                            break;
                        } else if !self.current().is_ascii_whitespace() || self.done() {
                            todo!("unclosed [")
                        }
                        self.step();
                    }
                    Target::ImmediateAddress(immediate)
                }
            }

            _ => {
                let word = self.take_id();
                if let Some(register) = Self::register_from_text(word) {
                    Target::Register(register)
                } else {
                    let immediate = Self::immediate_from_text(word);
                    Target::Immediate(immediate)
                }
            }
        }
    }
    fn take_id(&mut self) -> &[u8] {
        let word_start = self.cursor;
        let mut word_end = self.cursor;
        loop {
            self.step();
            if self.done()
                || !self.current().is_ascii_alphanumeric()
                    && self.current() != b'-'
                    && self.current() != b'_'
            {
                break;
            }
            word_end = self.cursor;
        }
        &self.inner[word_start..=word_end]
    }

    fn take_alphanumeric(&mut self) -> &[u8] {
        let word_start = self.cursor;
        let mut word_end = self.cursor;
        loop {
            self.step();
            if !self.current().is_ascii_alphanumeric() {
                break;
            }
            word_end = self.cursor;
        }
        &self.inner[word_start..=word_end]
    }
    fn parse_single(&mut self) -> Instruction {
        match self.current() {
            b';' => {
                self.skip_line();
                self.parse_single()
            }
            c if c.is_ascii_whitespace() => {
                self.step();
                self.parse_single()
            }
            c => {
                todo!("unhandled char '{}'", c as char);
            }
        }
    }
    pub fn parse(mut self) -> Vec<Instruction> {
        let mut instructions = Vec::new();
        loop {
            if self.done() {
                break;
            }
            instructions.push(self.parse_single());
        }
        instructions
    }
    pub fn new(inner: &'a [u8]) -> Self {
        Self {
            inner,
            character: 1,
            line: 1,
            cursor: 0,
        }
    }

    fn current(&self) -> u8 {
        self.inner[self.cursor]
    }

    fn done(&self) -> bool {
        self.cursor >= self.inner.len()
    }

    fn skip_whitespace(&mut self) {
        loop {
            if self.done() || !self.current().is_ascii_whitespace() {
                break;
            }
            self.step();
        }
    }

    fn skip_line(&mut self) {
        loop {
            if self.done() {
                break;
            }
            if self.current() == b'\n' {
                self.step();
                break;
            }
            self.step();
        }
    }

    fn step(&mut self) {
        if self.current() == b'\n' {
            self.line += 1;
            self.character = 1;
        } else {
            self.character += 1;
        }
        self.cursor += 1;
    }
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;

    use crate::{
        instructions::{Register, Target},
        Parser,
    };

    #[test]
    fn parse_target() {
        let mut parser = Parser::new(b"r0 r1 fl pc 4321");
        let r0 = parser.parse_target();
        assert_eq!(parser.current(), b' ');
        assert_eq!(Target::Register(Register::GeneralPurpose0), r0);
        parser.step();
        let r1 = parser.parse_target();
        assert_eq!(parser.current(), b' ');
        assert_eq!(Target::Register(Register::GeneralPurpose1), r1);
        parser.step();
        let fl = parser.parse_target();
        assert_eq!(parser.current(), b' ');
        assert_eq!(Target::Register(Register::Flag), fl);
        parser.step();
        let pc = parser.parse_target();
        assert_eq!(parser.current(), b' ');
        assert_eq!(Target::Register(Register::ProgramCounter), pc);
        parser.step();
        let imm = parser.parse_target();
        assert_eq!(Target::Immediate(4321), imm);
        assert!(parser.done());
    }

    #[test]
    fn parse_address_target() {
        let mut parser = Parser::new(b"[r0] [ r1 ] [fl] [pc] [ 4321 ]");

        let r0 = parser.parse_target();
        assert_eq!(Target::RegisterAddress(Register::GeneralPurpose0), r0);
        assert_eq!(parser.current(), b' ');
        parser.step();

        let r1 = parser.parse_target();
        assert_eq!(Target::RegisterAddress(Register::GeneralPurpose1), r1);
        assert_eq!(parser.current(), b' ');
        parser.step();

        let fl = parser.parse_target();
        assert_eq!(Target::RegisterAddress(Register::Flag), fl);
        assert_eq!(parser.current(), b' ');
        parser.step();

        let pc = parser.parse_target();
        assert_eq!(Target::RegisterAddress(Register::ProgramCounter), pc);
        assert_eq!(parser.current(), b' ');
        parser.step();

        let imm = parser.parse_target();
        assert_eq!(Target::ImmediateAddress(4321), imm);
        assert!(parser.done());
    }
}
