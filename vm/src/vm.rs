use crate::{
    arch::Word,
    named_instruction::{self, NamedInstruction},
};

pub type Immediate = crate::arch::Word;

pub struct Vm {
    memory: Vec<u8>,
    registers: VmRegisters,
    hlt_location: Option<Word>,
}

pub struct VmRegisters {
    general_purpose_0: Word,
    general_purpose_1: Word,
    flag: Word,
    program_counter: Word,
}

pub enum Flag {
    Overflow,
    CarryOrBorrow,
    Equal,
    Less,
    Below,
}

impl Flag {
    pub fn is_active(&self, value: u32) -> bool {
        match self {
            Flag::Overflow => (value & 0b0000_0001) != 0,
            Flag::CarryOrBorrow => (value & 0b0000_0010) >> 1 != 0,
            Flag::Equal => (value & 0b0000_0100) >> 2 != 0,
            Flag::Less => (value & 0b0000_1000) >> 3 != 0,
            Flag::Below => (value & 0b0001_0000) >> 4 != 0,
        }
    }
}

#[derive(Debug)]
pub enum Register {
    GeneralPurpose0,
    GeneralPurpose1,
    Flag,
    ProgramCounter,
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
    Register,
    Immediate,
    RegisterAddress,
    ImmediateAddress,
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

#[derive(Debug)]
pub enum Config {
    RegisterFromRegister(Register, Register),
    RegisterFromImmediate(Register, Immediate),
    RegisterFromRegisterAddress(Register, Register),
    RegisterFromImmediateAddress(Register, Immediate),
    RegisterAddressFromRegister(Register, Register),
    RegisterAddressFromImmediate(Register, Immediate),
    ImmediateAddressFromRegister(Immediate, Register),
    ImmediateAddressFromImmediate(Immediate, Immediate),
    ImmediateFromImmediate(Immediate, Immediate),
    ImmediateFromRegister(Immediate, Register),
}

#[derive(Debug)]
pub enum JmpConfig {
    Register(Register),
    Immediate(Immediate),
    RegisterAddress(Register),
    ImmediateAddress(Immediate),
}

#[derive(Debug)]
pub enum ConditionalJmpConfig {
    RegisterFromRegister(Register, Register),
    RegisterFromImmediate(Register, Immediate),
    RegisterFromRegisterAddress(Register, Register),
    RegisterFromImmediateAddress(Register, Immediate),
    ImmediateFromRegister(Immediate, Register),
    ImmediateFromImmediate(Immediate, Immediate),
    ImmediateFromRegisterAddress(Immediate, Register),
    ImmediateFromImmediateAddress(Immediate, Immediate),
    RegisterAddressFromRegister(Register, Register),
    RegisterAddressFromImmediate(Register, Immediate),
    ImmediateAddressFromRegister(Immediate, Register),
    ImmediateAddressFromImmediate(Immediate, Immediate),
}

#[derive(Debug)]
pub enum NotConfig {
    Register(Register),
    RegisterAddress(Register),
    ImmediateAddress(Immediate),
}

#[derive(Debug)]
pub enum Instruction {
    Nop,
    Hlt,
    Mov(Config),
    Not(NotConfig),
    Or(Config),
    And(Config),
    Xor(Config),
    Shl(Config),
    Shr(Config),
    Add(Config),
    Sub(Config),
    Mul(Config),
    IMul(Config),
    Div(Config),
    IDiv(Config),
    Rem(Config),
    Cmp(Config),
    Jmp(JmpConfig),
    Jz(ConditionalJmpConfig),
    Jnz(ConditionalJmpConfig),
}

pub enum MathOpVariant {
    Or,
    And,
    Xor,
    Shl,
    Shr,
    Mul,
    IMul,
    Div,
    IDiv,
    Rem,
}

pub fn invalid_architecture_message<E>(_error: E) -> String {
    String::from("architecture should support 32 bit word pointers")
}

#[derive(Debug)]
pub enum JmpVariant {
    Absolute,
    Relative,
}

pub enum ConditionalJmpVariant {
    Jz,
    Jnz,
}

impl Vm {
    pub fn new(instructions: Vec<u8>, memory_size: usize) -> Self {
        let mut memory = vec![0; memory_size];
        instructions
            .into_iter()
            .enumerate()
            .for_each(|(idx, byte)| memory[idx] = byte);
        Self {
            memory,
            hlt_location: None,
            registers: VmRegisters {
                general_purpose_0: 0,
                general_purpose_1: 0,
                flag: 0,
                program_counter: 0,
            },
        }
    }
    fn current_byte(&self) -> Result<u8, String> {
        self.registers
            .program_counter
            .try_into()
            .map_err(invalid_architecture_message)
            .map(|idx: usize| {
                self.memory.get(idx).ok_or_else(|| {
                    format!(
                        "cannot get current byte: index {idx} > {}",
                        self.memory.len()
                    )
                })
            })?
            .copied()
    }
    fn step(&mut self) {
        self.registers.program_counter += 1;
    }
    fn parse_conditional_jmp_target(
        &mut self,
        destination_selector: Selector,
        source_selector: Selector,
        destination: Result<Register, String>,
        source: Result<Register, String>,
    ) -> Result<ConditionalJmpConfig, String> {
        let config = match (destination_selector, source_selector) {
            (Selector::Register, Selector::Register) => {
                ConditionalJmpConfig::RegisterFromRegister(destination?, source?)
            }
            (Selector::Register, Selector::Immediate) => {
                ConditionalJmpConfig::RegisterFromImmediate(destination?, self.consume_immediate()?)
            }
            (Selector::Register, Selector::RegisterAddress) => {
                ConditionalJmpConfig::RegisterFromRegisterAddress(destination?, source?)
            }
            (Selector::Register, Selector::ImmediateAddress) => {
                ConditionalJmpConfig::RegisterFromImmediateAddress(
                    destination?,
                    self.consume_immediate()?,
                )
            }
            (Selector::RegisterAddress, Selector::Register) => {
                ConditionalJmpConfig::RegisterAddressFromRegister(destination?, source?)
            }
            (Selector::RegisterAddress, Selector::Immediate) => {
                ConditionalJmpConfig::RegisterAddressFromImmediate(
                    destination?,
                    self.consume_immediate()?,
                )
            }
            (Selector::ImmediateAddress, Selector::Register) => {
                ConditionalJmpConfig::ImmediateAddressFromRegister(
                    self.consume_immediate()?,
                    source?,
                )
            }
            (Selector::ImmediateAddress, Selector::Immediate) => {
                ConditionalJmpConfig::ImmediateAddressFromImmediate(
                    self.consume_immediate()?,
                    self.consume_immediate()?,
                )
            }
            (Selector::Immediate, Selector::Immediate) => {
                ConditionalJmpConfig::ImmediateFromImmediate(
                    self.consume_immediate()?,
                    self.consume_immediate()?,
                )
            }
            (Selector::Immediate, Selector::Register) => {
                ConditionalJmpConfig::ImmediateFromRegister(self.consume_immediate()?, source?)
            }
            (Selector::Immediate, Selector::RegisterAddress) => {
                ConditionalJmpConfig::ImmediateFromRegisterAddress(
                    self.consume_immediate()?,
                    source?,
                )
            }
            (Selector::Immediate, Selector::ImmediateAddress) => {
                ConditionalJmpConfig::ImmediateFromImmediateAddress(
                    self.consume_immediate()?,
                    self.consume_immediate()?,
                )
            }
            variant => Err(format!(
                "invalid selector/destination combo '{variant:?}' at {}",
                self.registers.program_counter
            ))?,
        };

        Ok(config)
    }

    fn parse_target(
        &mut self,
        destination_selector: Selector,
        source_selector: Selector,
        destination: Result<Register, String>,
        source: Result<Register, String>,
    ) -> Result<Config, String> {
        let config = match (destination_selector, source_selector) {
            (Selector::Register, Selector::Register) => {
                Config::RegisterFromRegister(destination?, source?)
            }
            (Selector::Register, Selector::Immediate) => {
                Config::RegisterFromImmediate(destination?, self.consume_immediate()?)
            }
            (Selector::Register, Selector::RegisterAddress) => {
                Config::RegisterFromRegisterAddress(destination?, source?)
            }
            (Selector::Register, Selector::ImmediateAddress) => {
                Config::RegisterFromImmediateAddress(destination?, self.consume_immediate()?)
            }
            (Selector::RegisterAddress, Selector::Register) => {
                Config::RegisterAddressFromRegister(destination?, source?)
            }
            (Selector::RegisterAddress, Selector::Immediate) => {
                Config::RegisterAddressFromImmediate(destination?, self.consume_immediate()?)
            }
            (Selector::ImmediateAddress, Selector::Register) => {
                Config::ImmediateAddressFromRegister(self.consume_immediate()?, source?)
            }
            (Selector::ImmediateAddress, Selector::Immediate) => {
                Config::ImmediateAddressFromImmediate(
                    self.consume_immediate()?,
                    self.consume_immediate()?,
                )
            }
            (Selector::Immediate, Selector::Immediate) => {
                Config::ImmediateFromImmediate(self.consume_immediate()?, self.consume_immediate()?)
            }
            (Selector::Immediate, Selector::Register) => {
                Config::ImmediateFromRegister(self.consume_immediate()?, source?)
            }
            variant => Err(format!(
                "invalid selector/destination combo '{variant:?}' at {}",
                self.registers.program_counter
            ))?,
        };

        Ok(config)
    }

    fn parse_mov(&mut self) -> Result<Instruction, String> {
        self.step();
        let input = self.current_byte()?;
        self.step();

        let destination_selector: Selector = ((input & 0b1100_0000) >> 6).try_into()?;
        let source_selector: Selector = ((input & 0b0011_0000) >> 4).try_into()?;

        let destination: Result<Register, _> = ((input & 0b0000_1100) >> 2).try_into();
        let source: Result<Register, _> = (input & 0b0000_0011).try_into();

        let instruction =
            self.parse_target(destination_selector, source_selector, destination, source)?;

        Ok(Instruction::Mov(instruction))
    }
    fn parse_nop(&mut self) -> Result<Instruction, String> {
        self.step();
        Ok(Instruction::Nop)
    }
    fn parse_hlt(&mut self) -> Result<Instruction, String> {
        self.step();
        Ok(Instruction::Hlt)
    }
    fn parse_not(&mut self) -> Result<Instruction, String> {
        self.step();
        let input = self.current_byte()?;
        self.step();

        let selector: Selector = ((input & 0b1100_0000) >> 6).try_into()?;
        let destination: Result<Register, _> = ((input & 0b0000_1100) >> 2).try_into();

        let config = match selector {
            Selector::Register => NotConfig::Register(destination?),
            Selector::Immediate => {
                return Err("invalid selector 'immediate' for not instruction".to_string())
            }
            Selector::RegisterAddress => NotConfig::RegisterAddress(destination?),
            Selector::ImmediateAddress => NotConfig::ImmediateAddress(self.consume_immediate()?),
        };

        Ok(Instruction::Not(config))
    }
    fn consume_immediate(&mut self) -> Result<u32, String> {
        let byte_0 = self.current_byte()?;
        self.step();
        let byte_1 = self.current_byte()?;
        self.step();
        let byte_2 = self.current_byte()?;
        self.step();
        let byte_3 = self.current_byte()?;
        self.step();
        Ok(u32::from_be_bytes([byte_0, byte_1, byte_2, byte_3]))
    }
    fn parse_jmp(&mut self) -> Result<Instruction, String> {
        self.step();
        let input = self.current_byte()?;
        self.step();

        log::debug!("parsing input '{input:#08b}'");

        let selector = ((input & 0b1100_0000) >> 6).try_into()?;
        let destination = ((input & 0b0000_1100) >> 2).try_into();

        let config = match selector {
            Selector::Register => JmpConfig::Register(destination?),
            Selector::Immediate => JmpConfig::Immediate(self.consume_immediate()?),
            Selector::RegisterAddress => JmpConfig::RegisterAddress(destination?),
            Selector::ImmediateAddress => JmpConfig::ImmediateAddress(self.consume_immediate()?),
        };

        Ok(Instruction::Jmp(config))
    }

    fn parse_conditional_jmp(&mut self) -> Result<Instruction, String> {
        let instruction: NamedInstruction = self.current_byte()?.try_into()?;
        self.step();
        let constructor = match instruction {
            named_instruction::Jz => Instruction::Jz,
            named_instruction::Jnz => Instruction::Jnz,
            field => Err(format!("invalid instruction {field:?}"))?,
        };

        let input = self.current_byte()?;
        self.step();

        log::debug!("parsing target ({input:#08b})");

        let destination_selector: Selector = ((input & 0b1100_0000) >> 6).try_into()?;
        let source_selector: Selector = ((input & 0b0011_0000) >> 4).try_into()?;
        let destination: Result<Register, _> = ((input & 0b0000_1100) >> 2).try_into();
        let source: Result<Register, _> = (input & 0b0000_0011).try_into();

        let config = self.parse_conditional_jmp_target(
            destination_selector,
            source_selector,
            destination,
            source,
        )?;

        Ok(constructor(config))
    }

    fn parse_math_op(&mut self) -> Result<Instruction, String> {
        let instruction: NamedInstruction = self.current_byte()?.try_into()?;
        self.step();
        let constructor = match instruction {
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
            instruction => unreachable!(
                "invalid instruction {instruction:?} at {:?}",
                self.registers.program_counter
            ),
        };

        let input = self.current_byte()?;
        self.step();

        let destination_selector = (input & 0b1100_0000) >> 6;
        let destination_selector: Selector = destination_selector.try_into()?;
        let source_selector = (input & 0b0011_0000) >> 4;
        let source_selector: Selector = source_selector.try_into()?;
        let destination = (input & 0b0000_1100) >> 2;
        let destination = destination.try_into();
        let source = input & 0b0000_0011;
        let source = source.try_into();

        let config =
            self.parse_target(destination_selector, source_selector, destination, source)?;

        Ok(constructor(config))
    }
    fn parse_next_instruction(&mut self) -> Result<Instruction, String> {
        let current_byte = self.current_byte()?;
        log::debug!("current byte: {current_byte:#02X}");
        let next: NamedInstruction = current_byte.try_into()?;
        log::debug!("parsing instruction {next:?} ({current_byte:#02X})");
        match next {
            named_instruction::Nop => self.parse_nop(),
            named_instruction::Hlt => self.parse_hlt(),
            named_instruction::Mov => self.parse_mov(),

            named_instruction::Or
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
            | named_instruction::Cmp => self.parse_math_op(),
            named_instruction::Not => self.parse_not(),
            named_instruction::Jmp => self.parse_jmp(),
            named_instruction::Jz | named_instruction::Jnz => self.parse_conditional_jmp(),
        }
    }
    pub fn register_value(&self, register: &Register) -> Word {
        match register {
            Register::GeneralPurpose0 => self.registers.general_purpose_0,
            Register::GeneralPurpose1 => self.registers.general_purpose_1,
            Register::Flag => self.registers.flag,
            Register::ProgramCounter => self.registers.program_counter,
        }
    }
    pub fn set_register_value(&mut self, register: &Register, value: Word) {
        match register {
            Register::GeneralPurpose0 => self.registers.general_purpose_0 = value,
            Register::GeneralPurpose1 => self.registers.general_purpose_1 = value,
            Register::Flag => self.registers.flag = value,
            Register::ProgramCounter => self.registers.program_counter = value,
        }
    }
    pub fn set_memory_value(&mut self, address: &Word, value: Word) -> Result<(), String> {
        let address: usize = (*address)
            .try_into()
            .map_err(invalid_architecture_message)?;

        value.to_be_bytes().into_iter().enumerate().try_for_each(
            |(offset, value)| -> Result<(), String> {
                let len = self.memory.len();
                let reference = self.memory.get_mut(address + offset).ok_or_else(|| {
                    format!(
                        "cannot get current byte: index {} > {len}",
                        address + offset,
                    )
                })?;
                *reference = value;
                Ok(())
            },
        )
    }
    pub fn memory_value(&self, address: &Word) -> Result<Word, String> {
        let address: usize = (*address)
            .try_into()
            .map_err(invalid_architecture_message)?;

        self.memory
            .get(address..address + 4)
            .ok_or_else(|| {
                format!(
                    "cannot get memory word: index {} > {}",
                    address + 4,
                    self.memory.len()
                )
            })
            .map(|bytes| u32::from_be_bytes(bytes.try_into().expect("grabbed 4 bytes")))
    }
    fn run_action_with_config<Action: FnOnce(Word, Word) -> Word>(
        &mut self,
        config: Config,
        action: Action,
    ) -> Result<(), String> {
        log::debug!("running action with config '{config:?}'");
        match config {
            Config::RegisterFromRegister(destination, source) => {
                let destination_value = self.register_value(&destination);
                let source_value = self.register_value(&source);
                self.set_register_value(&destination, action(destination_value, source_value))
            }
            Config::RegisterFromImmediate(destination, source) => {
                let destination_value = self.register_value(&destination);
                let source_value = source;
                self.set_register_value(&destination, action(destination_value, source_value))
            }
            Config::RegisterFromRegisterAddress(destination, source) => {
                let destination_value = self.register_value(&destination);
                let source_value = self.memory_value(&self.register_value(&source))?;
                self.set_register_value(&destination, action(destination_value, source_value))
            }
            Config::RegisterFromImmediateAddress(destination, source) => {
                let destination_value = self.register_value(&destination);
                let source_value = self.memory_value(&source)?;
                self.set_register_value(&destination, action(destination_value, source_value))
            }
            Config::RegisterAddressFromRegister(destination, source) => {
                let destination = self.register_value(&destination);
                let destination_value = self.memory_value(&destination)?;
                let source_value = self.register_value(&source);
                self.set_memory_value(&destination, action(destination_value, source_value))?
            }
            Config::RegisterAddressFromImmediate(destination, source) => {
                let destination = self.register_value(&destination);
                let destination_value = self.memory_value(&destination)?;
                let source_value = source;
                self.set_memory_value(&destination, action(destination_value, source_value))?
            }
            Config::ImmediateAddressFromRegister(destination, source) => {
                let destination_value = self.memory_value(&destination)?;
                let source_value = self.register_value(&source);
                self.set_memory_value(&destination, action(destination_value, source_value))?
            }
            Config::ImmediateAddressFromImmediate(destination, source) => {
                let destination_value = self.memory_value(&destination)?;
                let source_value = source;
                self.set_memory_value(&destination, action(destination_value, source_value))?
            }
            Config::ImmediateFromImmediate(destination, source) => {
                action(destination, source);
            }
            Config::ImmediateFromRegister(destination, source) => {
                let destination_value = destination;
                let source_value = self.register_value(&source);
                action(destination_value, source_value);
            }
        };
        Ok(())
    }
    fn run_mov(&mut self, config: Config) -> Result<(), String> {
        self.run_action_with_config(config, |_destination, source| source)
    }
    fn run_not(&mut self, config: NotConfig) -> Result<(), String> {
        match config {
            NotConfig::Register(register) => {
                self.set_register_value(&register, !self.register_value(&register));
            }
            NotConfig::RegisterAddress(register) => {
                let register_value = self.register_value(&register);
                let value = self.memory_value(&register_value)?;
                self.set_memory_value(&register_value, !value)?;
            }
            NotConfig::ImmediateAddress(immediate) => {
                let value = self.memory_value(&immediate)?;
                self.set_memory_value(&immediate, !value)?;
            }
        }
        Ok(())
    }
    fn run_cmp(&mut self, config: Config) -> Result<(), String> {
        let mut new_flag_value = None;

        self.run_action_with_config(config, |destination, source| {
            let flag_value = if destination == source { 0b100 } else { 0 };
            let flag_value = flag_value
                | if (destination as i32) < (source as i32) {
                    0b1000
                } else {
                    0
                };
            let flag_value = flag_value | if destination < source { 0b10000 } else { 0 };
            new_flag_value = Some(flag_value);
            destination
        })?;

        let Some(flag_value) = new_flag_value else {
            unreachable!("given closure should always run")
        };

        self.set_register_value(&Register::Flag, flag_value);

        Ok(())
    }
    fn run_sub(&mut self, config: Config) -> Result<(), String> {
        let flags = self.register_value(&Register::Flag);
        let carry_bit: Word = Flag::CarryOrBorrow.is_active(flags).into();

        let mut set_carry_bit = None;

        self.run_action_with_config(config, |destination, source| {
            let (result, overflowed) =
                (destination as i32).overflowing_sub_unsigned(source + carry_bit);
            set_carry_bit = Some(overflowed);
            result as u32
        })?;

        let flag_value = if let Some(set_carry_bit) = set_carry_bit {
            if set_carry_bit {
                flags | 0b10
            } else {
                flags & !0b10
            }
        } else {
            unreachable!("given closure should always run")
        };

        self.set_register_value(&Register::Flag, flag_value);
        Ok(())
    }

    fn run_add(&mut self, config: Config) -> Result<(), String> {
        let flags = self.register_value(&Register::Flag);
        let carry_bit: Word = Flag::CarryOrBorrow.is_active(flags).into();

        let mut set_carry_bit = None;

        self.run_action_with_config(config, |destination, source| {
            let (result, overflowed) =
                (destination as i32).overflowing_add_unsigned(source + carry_bit);
            set_carry_bit = Some(overflowed);
            result as u32
        })?;

        let flag_value = if let Some(set_carry_bit) = set_carry_bit {
            if set_carry_bit {
                flags | 0b10
            } else {
                flags & !0b10
            }
        } else {
            unreachable!("given closure should always run")
        };

        self.set_register_value(&Register::Flag, flag_value);
        Ok(())
    }
    fn run_conditional_jmp(
        &mut self,
        config: ConditionalJmpConfig,
        variant: ConditionalJmpVariant,
    ) -> Result<(), String> {
        let should_jump = match variant {
            ConditionalJmpVariant::Jz => |source| source == 0,
            ConditionalJmpVariant::Jnz => |source| source != 0,
        };
        let (destination, source) = match config {
            ConditionalJmpConfig::RegisterFromRegister(destination, source) => {
                let source = self.register_value(&source);
                let destination = self.register_value(&destination);
                (destination, source)
            }
            ConditionalJmpConfig::RegisterFromImmediate(destination, source) => {
                let destination = self.register_value(&destination);
                (destination, source)
            }
            ConditionalJmpConfig::RegisterFromRegisterAddress(destination, source) => {
                let source = self.memory_value(&self.register_value(&source))?;
                let destination = self.register_value(&destination);
                (destination, source)
            }
            ConditionalJmpConfig::RegisterFromImmediateAddress(destination, source) => {
                let source = self.memory_value(&source)?;
                let destination = self.register_value(&destination);
                (destination, source)
            }
            ConditionalJmpConfig::ImmediateFromRegister(destination, source) => {
                let source = self.register_value(&source);
                (destination, source)
            }
            ConditionalJmpConfig::ImmediateFromImmediate(destination, source) => {
                (destination, source)
            }
            ConditionalJmpConfig::ImmediateFromRegisterAddress(destination, source) => {
                let source = self.memory_value(&self.register_value(&source))?;
                (destination, source)
            }
            ConditionalJmpConfig::ImmediateFromImmediateAddress(destination, source) => {
                let source = self.memory_value(&source)?;
                (destination, source)
            }
            ConditionalJmpConfig::RegisterAddressFromRegister(destination, source) => {
                let source = self.register_value(&source);
                let destination = self.memory_value(&self.register_value(&destination))?;
                (destination, source)
            }
            ConditionalJmpConfig::RegisterAddressFromImmediate(destination, source) => {
                let destination = self.memory_value(&self.register_value(&destination))?;
                (destination, source)
            }
            ConditionalJmpConfig::ImmediateAddressFromRegister(destination, source) => {
                let source = self.register_value(&source);
                let destination = self.memory_value(&destination)?;
                (destination, source)
            }
            ConditionalJmpConfig::ImmediateAddressFromImmediate(destination, source) => {
                let destination = self.memory_value(&destination)?;
                (destination, source)
            }
        };

        if should_jump(source) {
            self.set_register_value(&Register::ProgramCounter, destination)
        }

        Ok(())
    }

    fn run_jmp(&mut self, config: JmpConfig) -> Result<(), String> {
        let destination = match config {
            JmpConfig::Register(register) => self.register_value(&register),
            JmpConfig::Immediate(immediate) => immediate,
            JmpConfig::RegisterAddress(register) => {
                self.memory_value(&self.register_value(&register))?
            }
            JmpConfig::ImmediateAddress(immediate) => self.memory_value(&immediate)?,
        };

        self.set_register_value(&Register::ProgramCounter, destination);

        log::debug!(
            "jmp: pc={:#04X} dest={destination:#04X}",
            self.register_value(&Register::ProgramCounter),
        );

        Ok(())
    }

    fn run_generic_math_op(
        &mut self,
        config: Config,
        variant: MathOpVariant,
    ) -> Result<(), String> {
        let action: fn(u32, u32) -> u32 = match variant {
            MathOpVariant::Or => std::ops::BitOr::bitor,
            MathOpVariant::And => std::ops::BitAnd::bitand,
            MathOpVariant::Xor => std::ops::BitXor::bitxor,
            MathOpVariant::Shl => std::ops::Shl::shl,
            MathOpVariant::Shr => std::ops::Shr::shr,
            MathOpVariant::Mul => std::ops::Mul::mul,
            MathOpVariant::IMul => |value, rhs| ((value as i32) * (rhs as i32)) as u32,
            MathOpVariant::Div => std::ops::Div::div,
            MathOpVariant::IDiv => |value, rhs| ((value as i32) / (rhs as i32)) as u32,
            MathOpVariant::Rem => std::ops::Rem::rem,
        };

        self.run_action_with_config(config, action)?;

        Ok(())
    }

    pub fn run_next_instruction(&mut self) -> Result<(), String> {
        if self.register_value(&Register::ProgramCounter) as usize >= self.memory.len() {
            return Err(String::from("out of instructions"));
        }
        let instruction_location = self.register_value(&Register::ProgramCounter);

        if let Some(hlt_location) = self.hlt_location {
            if instruction_location == hlt_location {
                return Ok(());
            }
            self.hlt_location = None;
        }

        log::debug!("parsing {instruction_location:#04X}",);
        let instruction = self.parse_next_instruction()?;
        log::debug!("running instruction {instruction:?} at {instruction_location:#04X}",);
        match instruction {
            Instruction::Nop => (),
            Instruction::Hlt => {
                self.hlt_location = Some(self.register_value(&Register::ProgramCounter));
            }
            Instruction::Mov(config) => self.run_mov(config)?,
            Instruction::Not(config) => self.run_not(config)?,
            Instruction::Or(config) => self.run_generic_math_op(config, MathOpVariant::Or)?,
            Instruction::And(config) => self.run_generic_math_op(config, MathOpVariant::And)?,
            Instruction::Xor(config) => self.run_generic_math_op(config, MathOpVariant::Xor)?,
            Instruction::Shl(config) => self.run_generic_math_op(config, MathOpVariant::Shl)?,
            Instruction::Shr(config) => self.run_generic_math_op(config, MathOpVariant::Shr)?,
            Instruction::Add(config) => self.run_add(config)?,
            Instruction::Sub(config) => self.run_sub(config)?,
            Instruction::Mul(config) => self.run_generic_math_op(config, MathOpVariant::Mul)?,
            Instruction::IMul(config) => self.run_generic_math_op(config, MathOpVariant::IMul)?,
            Instruction::Div(config) => self.run_generic_math_op(config, MathOpVariant::Div)?,
            Instruction::IDiv(config) => self.run_generic_math_op(config, MathOpVariant::IDiv)?,
            Instruction::Rem(config) => self.run_generic_math_op(config, MathOpVariant::Rem)?,
            Instruction::Cmp(config) => self.run_cmp(config)?,
            Instruction::Jmp(config) => self.run_jmp(config)?,
            Instruction::Jz(config) => {
                self.run_conditional_jmp(config, ConditionalJmpVariant::Jz)?
            }
            Instruction::Jnz(config) => {
                self.run_conditional_jmp(config, ConditionalJmpVariant::Jnz)?
            }
        }
        Ok(())
    }
}
