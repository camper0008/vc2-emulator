use std::borrow::Cow;

use crate::instructions::{
    instruction_from_text, Instruction, InstructionOrConstant, NamedInstruction,
    PreprocessorCommand, Register, Target,
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
                c => {
                    break Some(self.invalid_character_error(Cow::Owned(format!(
                        "expected end of line, got '{c}'"
                    ))))
                }
            }
        }
    }
    fn immediate_from_text(text: &[u8], from: Position, to: Position) -> Result<'a, u32> {
        let text = {
            let from = from.clone();
            let to = to.clone();
            let text = std::str::from_utf8(text).map_err(|_| Error {
                message: Cow::Borrowed("expected valid utf_8"),
                from: from.clone(),
                to: from.clone(),
            })?;
            if &text[0..1] == "'" {
                let mut bytes = text.bytes();
                let Some(start_quote) = bytes.next() else {
                    unreachable!("just asserted length to be >= 1");
                };
                assert_eq!(start_quote, b'\'');
                let Some(byte) = bytes.next() else {
                    return Err(Error {
                        message: Cow::Borrowed("unexpected end of char literal"),
                        from,
                        to,
                    });
                };
                let byte = if byte == b'\\' {
                    let Some(byte) = bytes.next() else {
                        return Err(Error {
                            message: Cow::Borrowed("unexpected end of char literal"),
                            from,
                            to,
                        });
                    };
                    Self::escaped_char_to_value(byte, from.clone(), to.clone())?
                } else {
                    byte
                };

                let Some(end_quote) = bytes.next() else {
                    return Err(Error {
                        message: Cow::Borrowed("unexpected end of char literal"),
                        from,
                        to,
                    });
                };

                if end_quote != b'\'' {
                    return Err(Error {
                        message: Cow::Owned(format!(
                            "expected single quote ('), got '{}'",
                            end_quote
                        )),
                        from,
                        to,
                    });
                }

                return Ok(byte.into());
            } else {
                text.replace('_', "")
            }
        };
        let value = if text.starts_with("0x") {
            i64::from_str_radix(&text[2..], 16)
        } else if text.starts_with("0b") {
            i64::from_str_radix(&text[2..], 2)
        } else {
            text.parse::<i64>()
        };

        let value = value.map_err(|_| {
            let from = from.clone();
            let to = to.clone();

            Error {
                message: Cow::Borrowed("invalid i64"),
                from,
                to,
            }
        })?;

        if value > i64::from(u32::MAX) {
            return Err(Error {
                message: Cow::Borrowed("number not within i32/u32 bounds"),
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
                Ok(Target::SubConstant(
                    String::from_utf8_lossy(&word[1..]).to_string(),
                ))
            } else {
                Ok(Target::Constant(String::from_utf8_lossy(word).to_string()))
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
                        return Err(self.invalid_character_error(Cow::Owned(format!(
                            "expected ']', got '{}'",
                            self.current() as char
                        ))));
                    };
                }
                Ok(match target {
                    Target::Register(register) => Target::RegisterAddress(register),
                    Target::Immediate(immediate) => Target::ImmediateAddress(immediate),
                    Target::Constant(label) => Target::ConstantAddress(label),
                    Target::SubConstant(label) => Target::SubConstantAddress(label),
                    Target::RegisterAddress(_)
                    | Target::ImmediateAddress(_)
                    | Target::ConstantAddress(_)
                    | Target::SubConstantAddress(_) => unreachable!(),
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
                    && self.current() != b'@'
                    && self.current() != b'\''
                    && self.current() != b'\\'
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
    fn invalid_character_error(&mut self, message: Cow<'a, str>) -> Error<'a> {
        let from = self.position();
        let to = self.position();
        self.skip_line();
        Error { message, from, to }
    }

    fn parse_instruction(
        &mut self,
        instruction: &NamedInstruction,
    ) -> Result<'a, InstructionOrConstant> {
        enum InstructionConstructor {
            None(Instruction),
            One(fn(Target) -> Instruction),
            Two(fn(Target, Target) -> Instruction),
        }
        let constructor = match instruction {
            NamedInstruction::Nop => InstructionConstructor::None(Instruction::Nop),
            NamedInstruction::Hlt => InstructionConstructor::None(Instruction::Hlt),
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
            NamedInstruction::Jmp => InstructionConstructor::One(|target| Instruction::Jmp(target)),
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
                    return Err(self.invalid_character_error(Cow::Owned(format!(
                        "expected character ':' for label, got '{}'",
                        self.current() as char
                    ))));
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
        Ok(InstructionOrConstant::Instruction(instruction))
    }
    fn parse_label(
        &mut self,
        text: &[u8],
        variant: &LabelVariant,
    ) -> Result<'a, InstructionOrConstant> {
        if self.current() != b':' {
            return Err(self.invalid_character_error(Cow::Owned(format!(
                "expected character ':' for label, got '{}'",
                self.current() as char
            ))));
        }
        self.step();
        let text = String::from_utf8_lossy(text).to_string();
        match variant {
            LabelVariant::Label => Ok(InstructionOrConstant::Label(text)),
            LabelVariant::SubLabel => Ok(InstructionOrConstant::SubLabel(text)),
        }
    }
    fn parse_db(&mut self) -> Result<'a, InstructionOrConstant> {
        let mut bytes = Vec::new();
        'db_loop: loop {
            let current = self.current();
            /* commas are optional */
            if current.is_ascii_whitespace() || self.current() == b',' {
                if current == b'\n' {
                    self.skip_whitespace();
                    break;
                }
                self.step();
                continue;
            };
            if self.current() == b'"' {
                'string_loop: loop {
                    self.step();
                    if self.done() || self.current() == b'\n' {
                        return Err(self.invalid_character_error(Cow::Borrowed(
                            "unexpected end of string literal",
                        )));
                    } else if self.current() == b'\\' {
                        self.step();
                        let from = self.position();
                        let to = self.position();
                        let char = self.current();
                        let byte = Self::escaped_char_to_value(char, from, to)?;
                        bytes.push(byte);
                        continue 'string_loop;
                    } else if self.current() == b'"' {
                        self.step();
                        continue 'db_loop;
                    }
                    bytes.push(self.current())
                }
            }
            let (text, from, to) = self.take_id();
            let byte = Self::immediate_from_text(text, from.clone(), to.clone())?;
            let byte = byte.try_into().map_err(|e| {
                log::debug!("error parsing u8 after db: {e}");
                Error {
                    from,
                    to,
                    message: Cow::Borrowed("expected u8, got value > 255"),
                }
            })?;
            bytes.push(byte);
        }
        Ok(InstructionOrConstant::PreprocessorCommand(
            PreprocessorCommand::DeclareBytes(bytes),
        ))
    }
    fn parse_dw(&mut self) -> Result<'a, InstructionOrConstant> {
        let (text, from, to) = self.take_id();
        let word = Self::immediate_from_text(text, from, to)?;
        if let Some(err) = self.ensure_no_dangling_arguments() {
            return Err(err);
        };
        Ok(InstructionOrConstant::PreprocessorCommand(
            PreprocessorCommand::DeclareWord(word),
        ))
    }
    fn parse_directive_or_instruction(&mut self) -> Result<'a, InstructionOrConstant> {
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
            None => match &id[..] {
                b"db" => {
                    self.step();
                    self.parse_db()
                }
                b"dw" => {
                    self.step();
                    self.parse_dw()
                }
                _ => self.parse_label(&id, &label_variant),
            },
        }
    }
    fn position(&self) -> Position {
        Position {
            cursor: self.cursor,
            line: self.line,
            character: self.character,
        }
    }
    fn parse_preprocessor_command(&mut self) -> Result<'a, InstructionOrConstant> {
        assert_eq!(
            self.current(),
            b'%',
            "should not be reached unless current is %"
        );
        self.step();
        let (id, from, to) = self.take_id();
        match id {
            b"offset_word" => {
                self.skip_whitespace();
                let (offset, from, to) = self.take_id();
                let offset = Self::immediate_from_text(offset, from, to)?;
                Ok(InstructionOrConstant::PreprocessorCommand(
                    PreprocessorCommand::Offset(offset * 4),
                ))
            }
            b"offset" => {
                self.skip_whitespace();
                let (offset, from, to) = self.take_id();
                let offset = Self::immediate_from_text(offset, from, to)?;
                Ok(InstructionOrConstant::PreprocessorCommand(
                    PreprocessorCommand::Offset(offset),
                ))
            }
            b"define" => {
                self.skip_whitespace();
                let (name, _, _) = self.take_id();
                let (name, cmd): (_, fn(_, _) -> PreprocessorCommand) = if name[0] == b'.' {
                    (&name[1..], PreprocessorCommand::DefineSub)
                } else {
                    (name, PreprocessorCommand::Define)
                };
                let name = String::from_utf8_lossy(name).to_string();
                self.skip_whitespace();
                let (value, from, to) = self.take_id();
                let offset = Self::immediate_from_text(value, from, to)?;
                Ok(InstructionOrConstant::PreprocessorCommand(cmd(
                    name, offset,
                )))
            }
            cmd => Err(Error {
                from,
                to,
                message: std::borrow::Cow::Owned(format!(
                    "unknown preprocessor command '{}'",
                    String::from_utf8_lossy(cmd)
                )),
            }),
        }
    }
    fn parse_single(&mut self) -> Result<'a, InstructionOrConstant> {
        if self.done() {
            return Ok(InstructionOrConstant::EOF);
        };
        log::debug!("current: {}", self.current() as char);
        match self.current() {
            b'%' => self.parse_preprocessor_command(),
            b';' => {
                self.skip_line();
                self.parse_single()
            }
            b'A'..=b'Z' | b'a'..=b'z' | b'.' => self.parse_directive_or_instruction(),
            c if c.is_ascii_whitespace() => {
                self.step();
                self.parse_single()
            }
            c => {
                let from = self.position();
                let to = self.position();
                self.step();
                Err(Error {
                    message: Cow::Owned(format!("unexpected character {c}")),
                    from,
                    to,
                })
            }
        }
    }
    #[must_use]
    pub fn parse(mut self) -> Vec<Result<'a, InstructionOrConstant>> {
        log::info!("parsing...");
        let mut instructions = Vec::new();
        loop {
            if self.done() {
                instructions.push(Ok(InstructionOrConstant::EOF));
                break;
            }
            instructions.push(self.parse_single())
        }
        log::info!("done");
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

    fn escaped_char_to_value(char: u8, from: Position, to: Position) -> Result<'a, u8> {
        let char = match char {
            b'n' => b'\n',
            b'r' => b'\r',
            b't' => b'\t',
            b'0' => b'\0',
            b'\\' => b'\\',
            b'\'' => b'\'',
            b'\"' => b'"',
            c => return Err(Error {
                message:
                    Cow::Owned(format!("unexpected escape char {c}, only \\0, \\n, \\r, \\t, \\' \\\" and \\\\ are supported.")),
                from,
                to,
            }),
        };
        Ok(char)
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
