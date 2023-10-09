use self::named_instruction::NamedInstruction;

mod named_instruction;

pub type Immediate = crate::arch::Word;

pub struct Parser<Iter: Iterator<Item = u8>> {
    feed: Iter,
}

#[repr(u8)]
pub enum Flag {
    Zero = 0,
    Less = 1,
    Equal = 2,
    Overflow = 3,
    Carry = 4,
    Borrow = 5,
}

#[repr(u8)]
pub enum Register {
    GeneralPurpose0 = 0b00,
    GeneralPurpose1 = 0b01,
    Flag = 0b10,
    ProgramCounter = 0b11,
}

impl TryFrom<u8> for Register {
    type Error = String;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0b00 => Ok(Self::GeneralPurpose0),
            0b01 => Ok(Self::GeneralPurpose1),
            0b10 => Ok(Self::Flag),
            0b11 => Ok(Self::ProgramCounter),
            value => Err(format!("invalid register 0b{value:b}")),
        }
    }
}

#[derive(Debug)]
#[repr(u8)]
pub enum Selector {
    Register = 0b00,
    Immediate = 0b01,
    RegisterAddress = 0b10,
    ImmediateAddress = 0b11,
}

impl TryFrom<u8> for Selector {
    type Error = String;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0b00 => Ok(Self::Register),
            0b01 => Ok(Self::Immediate),
            0b10 => Ok(Self::RegisterAddress),
            0b11 => Ok(Self::ImmediateAddress),
            value => Err(format!("invalid selector 0b{value:b}")),
        }
    }
}

pub enum Target {
    RegisterFromRegister(Register, Register),
    RegisterFromImmediate(Register, Immediate),
    RegisterFromRegisterAddress(Register, Register),
    RegisterFromImmediateAddress(Register, Immediate),
    RegisterAddressFromRegister(Register, Register),
    RegisterAddressFromImmediate(Register, Immediate),
    ImmediateAddressFromRegister(Immediate, Register),
    ImmediateAddressFromImmediate(Immediate, Immediate),
}

pub enum RegisterDestinationTarget {
    Register(Register, Register),
    Immediate(Register, Immediate),
    RegisterAddress(Register, Register),
    ImmediateAddress(Register, Immediate),
}

pub enum JmpTarget {
    Register(Register),
    Immediate(Immediate),
    RegisterAddress(Register),
    ImmediateAddress(Immediate),
}

pub enum Instruction {
    Htl,
    Mov(Target),
    Not(Register),
    Or(RegisterDestinationTarget),
    And(RegisterDestinationTarget),
    Xor(RegisterDestinationTarget),
    Shl(RegisterDestinationTarget),
    Shr(RegisterDestinationTarget),
    Add(RegisterDestinationTarget),
    Sub(RegisterDestinationTarget),
    Mul(RegisterDestinationTarget),
    IMul(RegisterDestinationTarget),
    Div(RegisterDestinationTarget),
    IDiv(RegisterDestinationTarget),
    Rem(RegisterDestinationTarget),
    Cmp(RegisterDestinationTarget),
    Jmp(JmpTarget),
    Jz(Target),
    Jnz(Target),
    Jeq(Target),
    Jne(Target),
    Jge(Target),
    Jgt(Target),
    Jle(Target),
    Jlt(Target),
}

type ParseResult = Result<Vec<Instruction>, String>;

impl<Iter: Iterator<Item = u8>> Parser<Iter> {
    pub fn new(feed: Iter) -> Self {
        Self { feed }
    }
    pub fn parse(self) -> ParseResult {
        self.parse_internal(Vec::new(), 0)
    }
    pub fn parse_target(
        &mut self,
        instruction_name: &str,
        op: usize,
        destination_selector: Selector,
        source_selector: Selector,
        destination: Result<Register, String>,
        source: Result<Register, String>,
    ) -> Result<(Target, usize), String> {
        macro_rules! consume_immediate {
            ($error: expr) => {
                u32::from_be_bytes([
                    self.feed.next().ok_or(format!($error))?,
                    self.feed.next().ok_or(format!($error))?,
                    self.feed.next().ok_or(format!($error))?,
                    self.feed.next().ok_or(format!($error))?,
                ])
            };
        }

        let (instruction, immediates_consumed) = match (destination_selector, source_selector) {
            (Selector::Register, Selector::Register) => {
                (Target::RegisterFromRegister(destination?, source?), 0)
            }
            (Selector::Register, Selector::Immediate) => (
                Target::RegisterFromImmediate(
                    destination?,
                    consume_immediate!(
                        "no immediate after '{instruction_name} reg imm' instruction at {op}"
                    ),
                ),
                1,
            ),
            (Selector::Register, Selector::RegisterAddress) => (
                Target::RegisterFromRegisterAddress(destination?, source?),
                0,
            ),
            (Selector::Register, Selector::ImmediateAddress) => (
                Target::RegisterFromImmediateAddress(
                    destination?,
                    consume_immediate!(
                        "no immediate after '{instruction_name} reg [imm]' instruction at {op}"
                    ),
                ),
                1,
            ),
            (Selector::RegisterAddress, Selector::Register) => (
                Target::RegisterAddressFromRegister(destination?, source?),
                0,
            ),
            (Selector::RegisterAddress, Selector::Immediate) => (
                Target::RegisterAddressFromImmediate(
                    destination?,
                    consume_immediate!(
                        "no immediate after '{instruction_name} [reg] imm' instruction at {op}"
                    ),
                ),
                1,
            ),
            (Selector::ImmediateAddress, Selector::Register) => (
                Target::ImmediateAddressFromRegister(
                    consume_immediate!(
                        "no immediate after '{instruction_name} [imm] reg' instruction at {op}"
                    ),
                    source?,
                ),
                1,
            ),
            (Selector::ImmediateAddress, Selector::Immediate) => (
                Target::ImmediateAddressFromImmediate(
                    consume_immediate!(
                        "no immediate after '{instruction_name} [imm] imm' instruction at {op}"
                    ),
                    consume_immediate!(
                        "no immediate after '{instruction_name} [imm] imm' instruction at {op}"
                    ),
                ),
                2,
            ),
            variant => Err(format!(
                "invalid selector/destination combo '{variant:?}' at {op}"
            ))?,
        };

        Ok((instruction, immediates_consumed))
    }

    fn parse_mov(
        mut self,
        mut instructions: Vec<Instruction>,
        op: usize,
    ) -> Result<Vec<Instruction>, String> {
        let input = self
            .feed
            .next()
            .ok_or(format!("no value after 'MOV' instruction at {op}"))?;

        let destination_selector: Selector = ((input & 0b1100_0000) >> 6).try_into()?;
        let source_selector: Selector = ((input & 0b0011_0000) >> 4).try_into()?;

        let destination: Result<Register, _> = ((input & 0b0000_1100) >> 2).try_into();
        let source: Result<Register, _> = (input & 0b0000_0011).try_into();

        let (instruction, immediates_consumed) = self.parse_target(
            "MOV",
            op,
            destination_selector,
            source_selector,
            destination,
            source,
        )?;

        instructions.push(Instruction::Mov(instruction));

        self.parse_internal(instructions, op + 1 + immediates_consumed * 4)
    }
    fn parse_htl(self, mut instructions: Vec<Instruction>, op: usize) -> ParseResult {
        instructions.push(Instruction::Htl);
        self.parse_internal(instructions, op)
    }
    fn parse_not(mut self, mut instructions: Vec<Instruction>, op: usize) -> ParseResult {
        let input = self
            .feed
            .next()
            .ok_or(format!("no value after 'NOT' instruction at {op}"))?;

        let destination = (input & 0b1100) >> 2;
        let destination = destination.try_into()?;
        instructions.push(Instruction::Not(destination));
        self.parse_internal(instructions, op + 1)
    }
    fn parse_jmp(mut self, mut instructions: Vec<Instruction>, op: usize) -> ParseResult {
        let input = self
            .feed
            .next()
            .ok_or(format!("no value after 'MOV' instruction at {op}"))?;

        let selector = ((input & 0b1100_0000) >> 6).try_into()?;
        let destination = ((input & 0b0000_1100) >> 2).try_into();

        macro_rules! consume_immediate {
            ($error: expr) => {
                u32::from_be_bytes([
                    self.feed.next().ok_or(format!($error))?,
                    self.feed.next().ok_or(format!($error))?,
                    self.feed.next().ok_or(format!($error))?,
                    self.feed.next().ok_or(format!($error))?,
                ])
            };
        }

        let (instruction, immediates_consumed) = match selector {
            Selector::Register => (JmpTarget::Register(destination?), 0),
            Selector::Immediate => (
                JmpTarget::Immediate(consume_immediate!(
                    "no immediate after 'JMP' instruction at {op}"
                )),
                1,
            ),
            Selector::RegisterAddress => (JmpTarget::RegisterAddress(destination?), 0),
            Selector::ImmediateAddress => (
                JmpTarget::ImmediateAddress(consume_immediate!(
                    "no immediate after 'JMP' instruction at {op}"
                )),
                1,
            ),
        };

        instructions.push(Instruction::Jmp(instruction));

        self.parse_internal(instructions, op + 1 + immediates_consumed * 4)
    }

    fn parse_conditional_jmp(
        mut self,
        field: NamedInstruction,
        mut instructions: Vec<Instruction>,
        op: usize,
    ) -> ParseResult {
        let name = format!("{field:?}");
        let constructor = match field {
            named_instruction::Jz => Instruction::Jz,
            named_instruction::Jnz => Instruction::Jnz,
            named_instruction::Jeq => Instruction::Jeq,
            named_instruction::Jne => Instruction::Jne,
            named_instruction::Jge => Instruction::Jge,
            named_instruction::Jgt => Instruction::Jgt,
            named_instruction::Jle => Instruction::Jle,
            named_instruction::Jlt => Instruction::Jlt,
            field => Err(format!("invalid instruction {field:?}"))?,
        };

        let input = self
            .feed
            .next()
            .ok_or(format!("no value after '{name}' instruction at {op}"))?;

        let destination_selector: Selector = ((input & 0b1100_0000) >> 6).try_into()?;
        let source_selector: Selector = ((input & 0b0011_0000) >> 4).try_into()?;
        let destination: Result<Register, _> = ((input & 0b0000_1100) >> 2).try_into();
        let source: Result<Register, _> = (input & 0b0000_0011).try_into();

        let (field, immediates_consumed) = self.parse_target(
            &name,
            op,
            destination_selector,
            source_selector,
            destination,
            source,
        )?;

        instructions.push(constructor(field));
        self.parse_internal(instructions, op + 1 + immediates_consumed)
    }

    fn parse_math_op(
        mut self,
        field: NamedInstruction,
        mut instructions: Vec<Instruction>,
        op: usize,
    ) -> ParseResult {
        let name = format!("{field:?}");
        let constructor = match field {
            named_instruction::Or => Instruction::Or,
            named_instruction::And => Instruction::And,
            named_instruction::Xor => Instruction::Xor,
            named_instruction::Shl => Instruction::Shl,
            named_instruction::Shr => Instruction::Shr,
            named_instruction::Add => Instruction::Add,
            named_instruction::Sub => Instruction::Sub,
            named_instruction::Mul => Instruction::Mul,
            named_instruction::IMul => Instruction::IMul,
            named_instruction::Div => Instruction::Div,
            named_instruction::IDiv => Instruction::IDiv,
            named_instruction::Rem => Instruction::Rem,
            named_instruction::Cmp => Instruction::Cmp,
            field => Err(format!("invalid instruction {field:?}"))?,
        };

        let input = self
            .feed
            .next()
            .ok_or(format!("no value after '{name}' instruction at {op}"))?;

        let selector = (input & 0b0011_0000) >> 4;
        let selector: Selector = selector.try_into()?;
        let destination = (input & 0b0000_1100) >> 2;
        let destination: Register = destination.try_into()?;
        let source = input & 0b0000_0011;
        let source: Register = source.try_into()?;

        macro_rules! consume_immediate {
            ($error: expr) => {
                u32::from_be_bytes([
                    self.feed.next().ok_or(format!($error))?,
                    self.feed.next().ok_or(format!($error))?,
                    self.feed.next().ok_or(format!($error))?,
                    self.feed.next().ok_or(format!($error))?,
                ])
            };
        }

        let (field, immediates_consumed) = match selector {
            Selector::Register => (RegisterDestinationTarget::Register(destination, source), 0),
            Selector::Immediate => (
                RegisterDestinationTarget::Immediate(
                    destination,
                    consume_immediate!("no immediate after '{name} reg imm' instruction at {op}"),
                ),
                1,
            ),
            Selector::RegisterAddress => (
                RegisterDestinationTarget::RegisterAddress(destination, source),
                0,
            ),
            Selector::ImmediateAddress => (
                RegisterDestinationTarget::ImmediateAddress(
                    destination,
                    consume_immediate!("no immediate after '{name} reg imm' instruction at {op}"),
                ),
                1,
            ),
        };

        instructions.push(constructor(field));

        self.parse_internal(instructions, op + 1 + immediates_consumed)
    }
    fn parse_internal(mut self, instructions: Vec<Instruction>, op: usize) -> ParseResult {
        let next: Option<Result<NamedInstruction, String>> =
            self.feed.next().map(TryInto::try_into);
        if let Some(Err(err)) = next {
            return Err(err);
        }
        let next = next.map(Result::unwrap);
        match next {
            Some(named_instruction::Nop) => self.parse_internal(instructions, op + 1),
            Some(named_instruction::Hlt) => self.parse_htl(instructions, op + 1),
            Some(named_instruction::Mov) => self.parse_mov(instructions, op + 1),
            Some(
                v @ (named_instruction::Or
                | named_instruction::And
                | named_instruction::Xor
                | named_instruction::Shl
                | named_instruction::Shr
                | named_instruction::Add
                | named_instruction::Sub
                | named_instruction::Mul
                | named_instruction::IMul
                | named_instruction::Div
                | named_instruction::IDiv
                | named_instruction::Rem
                | named_instruction::Cmp),
            ) => self.parse_math_op(v, instructions, op + 1),
            Some(named_instruction::Not) => self.parse_not(instructions, op + 1),
            Some(named_instruction::Jmp) => self.parse_jmp(instructions, op + 1),
            Some(
                v @ (named_instruction::Jz
                | named_instruction::Jnz
                | named_instruction::Jeq
                | named_instruction::Jne
                | named_instruction::Jge
                | named_instruction::Jgt
                | named_instruction::Jle
                | named_instruction::Jlt),
            ) => self.parse_conditional_jmp(v, instructions, op + 1),
            None => Ok(instructions),
        }
    }
}
