#[derive(Debug)]
pub enum NamedInstruction {
    Nop,
    Hlt,
    Mov,
    Or,
    And,
    Xor,
    Not,
    Shl,
    Shr,
    Add,
    Sub,
    Mul,
    IMul,
    Div,
    IDiv,
    Rem,
    Cmp,
    Jmp,
    Jz,
    Jnz,
}

pub use NamedInstruction::*;

impl TryFrom<u8> for NamedInstruction {
    type Error = String;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(NamedInstruction::Nop),
            0x01 => Ok(NamedInstruction::Hlt),
            0x02 => Ok(NamedInstruction::Mov),
            0x03 => Ok(NamedInstruction::Or),
            0x04 => Ok(NamedInstruction::And),
            0x05 => Ok(NamedInstruction::Xor),
            0x06 => Ok(NamedInstruction::Not),
            0x07 => Ok(NamedInstruction::Shl),
            0x08 => Ok(NamedInstruction::Shr),
            0x09 => Ok(NamedInstruction::Add),
            0x0A => Ok(NamedInstruction::Sub),
            0x0B => Ok(NamedInstruction::Mul),
            0x0C => Ok(NamedInstruction::IMul),
            0x0D => Ok(NamedInstruction::Div),
            0x0E => Ok(NamedInstruction::IDiv),
            0x0F => Ok(NamedInstruction::Rem),
            0x10 => Ok(NamedInstruction::Cmp),
            0x11 => Ok(NamedInstruction::Jmp),
            0x12 => Ok(NamedInstruction::Jz),
            0x13 => Ok(NamedInstruction::Jnz),
            instruction => Err(format!("unrecognized instruction '0x{instruction:X}'")),
        }
    }
}
