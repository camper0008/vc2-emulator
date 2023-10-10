use std::time::Duration;

use crate::{
    arch::Word,
    named_instruction::{self, NamedInstruction},
};

pub type Immediate = crate::arch::Word;

pub struct Vm<const MEMORY_BYTE_SIZE: usize, const HALT_MS: u64> {
    memory: [u8; MEMORY_BYTE_SIZE],
    registers: VmRegisters,
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
}

impl Flag {
    pub fn is_active(&self, value: u32) -> bool {
        match self {
            Flag::Overflow => (value & 0b0000_0001) != 0,
            Flag::CarryOrBorrow => (value & 0b0000_0010) >> 1 != 0,
            Flag::Equal => (value & 0b0000_0100) >> 2 != 0,
            Flag::Less => (value & 0b0000_1000) >> 3 != 0,
        }
    }
}

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

pub enum Config {
    RegisterFromRegister(Register, Register),
    RegisterFromImmediate(Register, Immediate),
    RegisterFromRegisterAddress(Register, Register),
    RegisterFromImmediateAddress(Register, Immediate),
    RegisterAddressFromRegister(Register, Register),
    RegisterAddressFromImmediate(Register, Immediate),
    ImmediateAddressFromRegister(Immediate, Register),
    ImmediateAddressFromImmediate(Immediate, Immediate),
}

pub enum JmpConfig {
    Register(Register),
    Immediate(Immediate),
    RegisterAddress(Register),
    ImmediateAddress(Immediate),
}

pub enum Instruction {
    Nop,
    Htl,
    Mov(Config),
    Not(Register),
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
    Jmp(bool, JmpConfig),
    Jz(Config),
    Jnz(Config),
    Jeq(Config),
    Jne(Config),
    Jge(Config),
    Jgt(Config),
    Jle(Config),
    Jlt(Config),
}

pub fn invalid_architecture_message<E>(_error: E) -> String {
    String::from("architecture should support 32 bit word pointers")
}

impl<const MEMORY_BYTE_SIZE: usize, const HALT_MS: u64> Vm<MEMORY_BYTE_SIZE, HALT_MS> {
    pub fn new(instructions: Vec<u8>) -> Self {
        let mut memory = [0; MEMORY_BYTE_SIZE];
        instructions
            .into_iter()
            .enumerate()
            .for_each(|(idx, byte)| memory[idx] = byte);
        Self {
            memory,
            registers: VmRegisters {
                general_purpose_0: 0,
                general_purpose_1: 0,
                flag: 0,
                program_counter: 0,
            },
        }
    }
    pub fn run(mut self) -> Result<(), String> {
        loop {
            self.run_next_instruction()?;
        }
    }
    fn current_byte(&self) -> Result<u8, String> {
        self.registers
            .program_counter
            .try_into()
            .map(|idx: usize| self.memory[idx])
            .map_err(invalid_architecture_message)
    }
    fn step(&mut self) {
        self.registers.program_counter += 1;
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
    fn parse_htl(&mut self) -> Result<Instruction, String> {
        self.step();
        Ok(Instruction::Htl)
    }
    fn parse_not(&mut self) -> Result<Instruction, String> {
        self.step();
        let input = self.current_byte()?;
        self.step();

        let destination = (input & 0b1100) >> 2;
        let destination = destination.try_into()?;
        Ok(Instruction::Not(destination))
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

        let selector = ((input & 0b1100_0000) >> 6).try_into()?;
        let destination = ((input & 0b0000_1100) >> 2).try_into();
        let is_absolute = (input & 0b0000_0001) != 0;

        let config = match selector {
            Selector::Register => JmpConfig::Register(destination?),
            Selector::Immediate => JmpConfig::Immediate(self.consume_immediate()?),
            Selector::RegisterAddress => JmpConfig::RegisterAddress(destination?),
            Selector::ImmediateAddress => JmpConfig::ImmediateAddress(self.consume_immediate()?),
        };

        Ok(Instruction::Jmp(is_absolute, config))
    }

    fn parse_conditional_jmp(&mut self) -> Result<Instruction, String> {
        let instruction: NamedInstruction = self.current_byte()?.try_into()?;
        self.step();
        let constructor = match instruction {
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

        let input = self.current_byte()?;
        self.step();

        let destination_selector: Selector = ((input & 0b1100_0000) >> 6).try_into()?;
        let source_selector: Selector = ((input & 0b0011_0000) >> 4).try_into()?;
        let destination: Result<Register, _> = ((input & 0b0000_1100) >> 2).try_into();
        let source: Result<Register, _> = (input & 0b0000_0011).try_into();

        let config =
            self.parse_target(destination_selector, source_selector, destination, source)?;

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
        let next: NamedInstruction = self.current_byte()?.try_into()?;
        match next {
            named_instruction::Nop => self.parse_nop(),
            named_instruction::Hlt => self.parse_htl(),
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

            named_instruction::Jz
            | named_instruction::Jnz
            | named_instruction::Jeq
            | named_instruction::Jne
            | named_instruction::Jge
            | named_instruction::Jgt
            | named_instruction::Jle
            | named_instruction::Jlt => self.parse_conditional_jmp(),
        }
    }
    fn register_value(&self, register: &Register) -> Word {
        match register {
            Register::GeneralPurpose0 => self.registers.general_purpose_0,
            Register::GeneralPurpose1 => self.registers.general_purpose_1,
            Register::Flag => self.registers.flag,
            Register::ProgramCounter => self.registers.program_counter,
        }
    }
    fn set_register_value(&mut self, register: &Register, value: Word) {
        match register {
            Register::GeneralPurpose0 => self.registers.general_purpose_0 = value,
            Register::GeneralPurpose1 => self.registers.general_purpose_0 = value,
            Register::Flag => self.registers.general_purpose_0 = value,
            Register::ProgramCounter => self.registers.general_purpose_0 = value,
        }
    }
    fn set_memory_value(&mut self, address: &Word, value: Word) -> Result<(), String> {
        let address: usize = (address * 4)
            .try_into()
            .map_err(invalid_architecture_message)?;

        value
            .to_be_bytes()
            .into_iter()
            .enumerate()
            .for_each(|(offset, value)| self.memory[address + offset] = value);

        Ok(())
    }
    fn memory_value(&self, address: &Word) -> Result<Word, String> {
        let address: usize = (address * 4)
            .try_into()
            .map_err(invalid_architecture_message)?;

        let bytes = &self.memory[address..address + 4];
        Ok(u32::from_be_bytes(
            bytes.try_into().expect("grabbed 4 bytes"),
        ))
    }
    fn run_action_with_config<Action: FnOnce(Word, Word) -> Word>(
        &mut self,
        config: Config,
        action: Action,
    ) -> Result<(), String> {
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
        };
        Ok(())
    }
    fn run_mov(&mut self, config: Config) -> Result<(), String> {
        self.run_action_with_config(config, |_destination, source| source)
    }
    fn run_not(&mut self, config: Register) {
        self.set_register_value(&config, !self.register_value(&config))
    }
    fn run_cmp(&mut self, config: Config) -> Result<(), String> {
        let flags = self.register_value(&Register::Flag);
        let compare = match (Flag::Equal.is_active(flags), Flag::Less.is_active(flags)) {
            (true, true) => PartialOrd::<Word>::le,
            (true, false) => PartialEq::<Word>::eq,
            (false, true) => PartialOrd::<Word>::lt,
            (false, false) => PartialEq::<Word>::ne,
        };
        self.run_action_with_config(config, |destination, source| {
            compare(&destination, &source).into()
        })
    }
    fn run_add(&mut self, config: Config) -> Result<(), String> {
        let flags = self.register_value(&Register::Flag);
        let carry_bit: Word = Flag::CarryOrBorrow.is_active(flags).into();

        /* borrow checker hack :S */
        let mut set_carry_bit = None;

        self.run_action_with_config(config, |destination, source| {
            let result = destination.checked_add(source + carry_bit);
            if let Some(destination_value) = result {
                set_carry_bit = Some(false);
                destination_value
            } else {
                set_carry_bit = Some(true);
                0
            }
        })?;

        let flag_value = if let Some(result) = set_carry_bit {
            if result {
                flags & !0b10
            } else {
                flags | 0b10
            }
        } else {
            unreachable!("given closure should always run")
        };

        self.set_register_value(&Register::Flag, flag_value);
        Ok(())
    }
    fn run_jnz(&mut self, config: Config) -> Result<(), String> {
        let mut destination_value = None;
        self.run_action_with_config(config, |destination, source| {
            if source != 0 {
                destination_value = Some(destination);
            } else {
                destination_value = Some(0);
            }
            destination
        })?;
        if let Some(offset) = destination_value {
            self.set_register_value(
                &Register::ProgramCounter,
                self.register_value(&Register::ProgramCounter)
                    .wrapping_add_signed(offset as i32),
            );
        } else {
            unreachable!("given closure should always run")
        }

        Ok(())
    }

    pub fn run_next_instruction(&mut self) -> Result<(), String> {
        let instruction = self.parse_next_instruction()?;
        match instruction {
            Instruction::Nop => (),
            Instruction::Htl => std::thread::sleep(Duration::from_millis(HALT_MS)),
            Instruction::Mov(config) => self.run_mov(config)?,
            Instruction::Not(config) => self.run_not(config),
            Instruction::Or(_) => todo!(),
            Instruction::And(_) => todo!(),
            Instruction::Xor(_) => todo!(),
            Instruction::Shl(_) => todo!(),
            Instruction::Shr(_) => todo!(),
            Instruction::Add(config) => self.run_add(config)?,
            Instruction::Sub(_) => todo!(),
            Instruction::Mul(_) => todo!(),
            Instruction::IMul(_) => todo!(),
            Instruction::Div(_) => todo!(),
            Instruction::IDiv(_) => todo!(),
            Instruction::Rem(_) => todo!(),
            Instruction::Cmp(config) => self.run_cmp(config)?,
            Instruction::Jmp(_, _) => todo!(),
            Instruction::Jz(_) => todo!(),
            Instruction::Jnz(config) => self.run_jnz(config)?,
            Instruction::Jeq(_) => todo!(),
            Instruction::Jne(_) => todo!(),
            Instruction::Jge(_) => todo!(),
            Instruction::Jgt(_) => todo!(),
            Instruction::Jle(_) => todo!(),
            Instruction::Jlt(_) => todo!(),
        }
        Ok(())
    }
}
