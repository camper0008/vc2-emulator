use crate::instructions::{
    instruction_from_text, Instruction, InstructionOrLabel, JmpVariant, NamedInstruction, Register,
    Target,
};

use crate::error::{Error, Position, Result};

pub struct Parser<'a> {
    cursor: usize,
    character: usize,
    line: usize,
    inner: &'a [u8],
}

enum LabelVariant {
    Label,
    SubLabel,
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
    #[must_use]
    fn ensure_no_dangling_arguments(&mut self) -> Option<Error<'a>> {
        loop {
            if self.done() {
                break None;
            }
            match self.current() {
                b'\n' => {
                    break None;
                }
                b';' => {
                    self.skip_line();
                    break None;
                }
                c if c.is_ascii_whitespace() => {
                    self.step();
                }
                _ => break Some(self.invalid_character_error("expected end of line, got")),
            }
        }
    }
    fn immediate_from_text(value: &[u8], from: Position, to: Position) -> Result<'a, u32> {
        let value = if value.starts_with(b"0x") {
            let from = from.clone();
            let to = to.clone();
            let value = &value[2..];
            i64::from_str_radix(
                std::str::from_utf8(value).map_err(|_| Error {
                    message: "expected valid utf_8",
                    from,
                    to,
                })?,
                16,
            )
        } else if value.starts_with(b"0b") {
            let from = from.clone();
            let to = to.clone();
            let value = &value[2..];
            i64::from_str_radix(
                std::str::from_utf8(value).map_err(|_| Error {
                    message: "expected valid utf_8",
                    from,
                    to,
                })?,
                2,
            )
        } else {
            let from = from.clone();
            let to = to.clone();
            std::str::from_utf8(value)
                .map_err(|_| Error {
                    message: "expected valid utf_8",
                    from,
                    to,
                })?
                .parse::<i64>()
        };

        let value = value.map_err(|_| {
            let from = from.clone();
            let to = to.clone();

            Error {
                message: "invalid i64",
                from,
                to,
            }
        })?;

        if (value < 0 && (value > i64::from(i32::MAX) || value < i64::from(i32::MIN)))
            || value > i64::from(u32::MAX)
        {
            return Err(Error {
                message: "number not within i32/u32 bounds",
                from,
                to,
            });
        }
        Ok(value as u32)
    }
    fn parse_target_literal(&mut self) -> Result<'a, Target> {
        self.skip_whitespace();
        let (word, from, to) = self.take_id();
        if let Some(register) = Self::register_from_text(word) {
            Ok(Target::Register(register))
        } else if !word[0].is_ascii_digit() {
            if word[0] == b'.' {
                Ok(Target::SubLabel(
                    String::from_utf8_lossy(&word[1..]).to_string(),
                ))
            } else {
                Ok(Target::Label(String::from_utf8_lossy(word).to_string()))
            }
        } else {
            let immediate = Self::immediate_from_text(word, from, to)?;
            Ok(Target::Immediate(immediate))
        }
    }
    fn parse_target(&mut self) -> Result<'a, Target> {
        self.skip_whitespace();
        match self.current() {
            b'[' => {
                self.step();
                let target = self.parse_target_literal()?;
                loop {
                    if self.current() == b']' {
                        self.step();
                        break;
                    } else if self.current().is_ascii_whitespace() {
                        self.step();
                    } else {
                        return Err(self.invalid_character_error("expected ']'"));
                    };
                }
                Ok(match target {
                    Target::Register(register) => Target::RegisterAddress(register),
                    Target::Immediate(immediate) => Target::ImmediateAddress(immediate),
                    Target::RegisterAddress(_)
                    | Target::ImmediateAddress(_)
                    | Target::Label(_)
                    | Target::SubLabel(_) => unreachable!(),
                })
            }
            _ => self.parse_target_literal(),
        }
    }
    fn take_id(&mut self) -> (&[u8], Position, Position) {
        let word_start = self.position();
        let mut word_end = self.position();
        loop {
            self.step();
            if self.done()
                || !self.current().is_ascii_alphanumeric()
                    && self.current() != b'-'
                    && self.current() != b'_'
            {
                break;
            }
            word_end = self.position();
        }
        (
            &self.inner[word_start.cursor..=word_end.cursor],
            word_start,
            word_end,
        )
    }
    fn invalid_character_error(&mut self, message: &'a str) -> Error<'a> {
        let from = self.position();
        let to = self.position();
        self.skip_line();
        Error { message, from, to }
    }

    fn parse_instruction(
        &mut self,
        instruction: &NamedInstruction,
    ) -> Result<'a, InstructionOrLabel> {
        enum InstructionConstructor {
            None(Instruction),
            One(fn(Target) -> Instruction),
            Two(fn(Target, Target) -> Instruction),
        }
        let constructor = match instruction {
            NamedInstruction::Nop => InstructionConstructor::None(Instruction::Nop),
            NamedInstruction::Htl => InstructionConstructor::None(Instruction::Htl),
            NamedInstruction::Mov => InstructionConstructor::Two(Instruction::Mov),
            NamedInstruction::Not => InstructionConstructor::One(Instruction::Not),
            NamedInstruction::Or => InstructionConstructor::Two(Instruction::Or),
            NamedInstruction::And => InstructionConstructor::Two(Instruction::And),
            NamedInstruction::Xor => InstructionConstructor::Two(Instruction::Xor),
            NamedInstruction::Shl => InstructionConstructor::Two(Instruction::Shl),
            NamedInstruction::Shr => InstructionConstructor::Two(Instruction::Shr),
            NamedInstruction::Add => InstructionConstructor::Two(Instruction::Add),
            NamedInstruction::Sub => InstructionConstructor::Two(Instruction::Sub),
            NamedInstruction::Mul => InstructionConstructor::Two(Instruction::Mul),
            NamedInstruction::IMul => InstructionConstructor::Two(Instruction::IMul),
            NamedInstruction::Div => InstructionConstructor::Two(Instruction::Div),
            NamedInstruction::IDiv => InstructionConstructor::Two(Instruction::IDiv),
            NamedInstruction::Rem => InstructionConstructor::Two(Instruction::Rem),
            NamedInstruction::Cmp => InstructionConstructor::Two(Instruction::Cmp),
            NamedInstruction::Jmp => {
                InstructionConstructor::One(|target| Instruction::Jmp(target, JmpVariant::Relative))
            }
            NamedInstruction::JmpAbs => {
                InstructionConstructor::One(|target| Instruction::Jmp(target, JmpVariant::Absolute))
            }
            NamedInstruction::Jz => InstructionConstructor::Two(Instruction::Jz),
            NamedInstruction::Jnz => InstructionConstructor::Two(Instruction::Jnz),
        };
        let instruction = match constructor {
            InstructionConstructor::None(instruction) => instruction,
            InstructionConstructor::One(constructor) => constructor(self.parse_target()?),
            InstructionConstructor::Two(constructor) => {
                let target_0 = self.parse_target()?;
                self.skip_whitespace();
                if self.current() != b',' {
                    return Err(self.invalid_character_error("expected character ',', got"));
                }
                self.step();
                self.skip_whitespace();
                let target_1 = self.parse_target()?;
                constructor(target_0, target_1)
            }
        };
        if let Some(err) = self.ensure_no_dangling_arguments() {
            return Err(err);
        }
        Ok(InstructionOrLabel::Instruction(instruction))
    }
    fn parse_label(
        &mut self,
        text: &[u8],
        variant: &LabelVariant,
    ) -> Result<'a, InstructionOrLabel> {
        if self.current() != b':' {
            return Err(self.invalid_character_error("expected character ':' for label, got"));
        }
        self.step();
        if let Some(err) = self.ensure_no_dangling_arguments() {
            return Err(err);
        }
        let text = String::from_utf8_lossy(text).to_string();
        match variant {
            LabelVariant::Label => Ok(InstructionOrLabel::Label(text)),
            LabelVariant::SubLabel => Ok(InstructionOrLabel::SubLabel(text)),
        }
    }
    fn parse_label_or_instruction(&mut self) -> Result<'a, InstructionOrLabel> {
        let label_variant = if self.current() == b'.' {
            self.step();
            LabelVariant::SubLabel
        } else {
            LabelVariant::Label
        };
        let (id, _, _) = self.take_id();
        let id = id.to_vec();
        match instruction_from_text(&id) {
            Some(instruction) => self.parse_instruction(&instruction),
            None => self.parse_label(&id, &label_variant),
        }
    }
    fn position(&self) -> Position {
        Position {
            cursor: self.cursor,
            line: self.line,
            character: self.character,
        }
    }
    fn parse_single(&mut self) -> Result<'a, InstructionOrLabel> {
        if self.done() {
            return Ok(InstructionOrLabel::EOF);
        };
        match self.current() {
            b';' => {
                self.skip_line();
                self.parse_single()
            }
            b'A'..=b'Z' | b'a'..=b'z' | b'.' => self.parse_label_or_instruction(),
            c if c.is_ascii_whitespace() => {
                self.step();
                self.parse_single()
            }
            _ => {
                let from = self.position();
                let to = self.position();
                Err(Error {
                    message: "unexpected character",
                    from,
                    to,
                })
            }
        }
    }
    #[must_use]
    pub fn parse(mut self) -> Vec<Result<'a, InstructionOrLabel>> {
        let mut instructions = Vec::new();
        loop {
            if self.done() {
                instructions.push(Ok(InstructionOrLabel::EOF));
                break;
            }
            instructions.push(self.parse_single());
        }
        instructions
    }
    #[must_use]
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
        let r0 = parser.parse_target().unwrap();
        assert_eq!(parser.current(), b' ');
        assert_eq!(Target::Register(Register::GeneralPurpose0), r0);
        parser.step();
        let r1 = parser.parse_target().unwrap();
        assert_eq!(parser.current(), b' ');
        assert_eq!(Target::Register(Register::GeneralPurpose1), r1);
        parser.step();
        let fl = parser.parse_target().unwrap();
        assert_eq!(parser.current(), b' ');
        assert_eq!(Target::Register(Register::Flag), fl);
        parser.step();
        let pc = parser.parse_target().unwrap();
        assert_eq!(parser.current(), b' ');
        assert_eq!(Target::Register(Register::ProgramCounter), pc);
        parser.step();
        let imm = parser.parse_target().unwrap();
        assert_eq!(Target::Immediate(4321), imm);
        assert!(parser.done());
    }

    #[test]
    fn parse_address_target() {
        let mut parser = Parser::new(b"[r0] [ r1 ] [fl] [pc] [ 4321 ]");

        let r0 = parser.parse_target().unwrap();
        assert_eq!(Target::RegisterAddress(Register::GeneralPurpose0), r0);
        assert_eq!(parser.current(), b' ');
        parser.step();

        let r1 = parser.parse_target().unwrap();
        assert_eq!(Target::RegisterAddress(Register::GeneralPurpose1), r1);
        assert_eq!(parser.current(), b' ');
        parser.step();

        let fl = parser.parse_target().unwrap();
        assert_eq!(Target::RegisterAddress(Register::Flag), fl);
        assert_eq!(parser.current(), b' ');
        parser.step();

        let pc = parser.parse_target().unwrap();
        assert_eq!(Target::RegisterAddress(Register::ProgramCounter), pc);
        assert_eq!(parser.current(), b' ');
        parser.step();

        let imm = parser.parse_target().unwrap();
        assert_eq!(Target::ImmediateAddress(4321), imm);
        assert!(parser.done());
    }
}
