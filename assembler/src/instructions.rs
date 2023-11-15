#[derive(Debug, PartialEq, Clone)]
pub enum Target {
    Register(Register),
    RegisterAddress(Register),
    Immediate(Immediate),
    ImmediateAddress(Immediate),
    Label(String),
    SubLabel(String),
}

pub type Immediate = u32;

#[derive(Debug, PartialEq, Clone)]
pub enum Register {
    GeneralPurpose0,
    GeneralPurpose1,
    Flag,
    ProgramCounter,
}

#[derive(Debug, Clone)]
pub enum InstructionOrLabel {
    Instruction(Instruction),
    Label(String),
    SubLabel(String),
    EOF,
}

#[derive(Debug, Clone)]
pub enum Instruction {
    Nop,
    Htl,
    Mov(Target, Target),
    Not(Target),
    Or(Target, Target),
    And(Target, Target),
    Xor(Target, Target),
    Shl(Target, Target),
    Shr(Target, Target),
    Add(Target, Target),
    Sub(Target, Target),
    Mul(Target, Target),
    IMul(Target, Target),
    Div(Target, Target),
    IDiv(Target, Target),
    Rem(Target, Target),
    Cmp(Target, Target),
    Jmp(Target, JmpVariant),
    Jz(Target, Target),
    Jnz(Target, Target),
}

#[derive(Debug)]
pub enum NamedInstruction {
    Nop,
    Htl,
    Mov,
    Not,
    Or,
    And,
    Xor,
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
    JmpAbs,
    Jz,
    Jnz,
}

pub fn instruction_from_text(text: &[u8]) -> Option<NamedInstruction> {
    match text {
        b"nop" => Some(NamedInstruction::Nop),
        b"htl" => Some(NamedInstruction::Htl),
        b"mov" => Some(NamedInstruction::Mov),
        b"not" => Some(NamedInstruction::Not),
        b"or" => Some(NamedInstruction::Or),
        b"and" => Some(NamedInstruction::And),
        b"xor" => Some(NamedInstruction::Xor),
        b"shl" => Some(NamedInstruction::Shl),
        b"shr" => Some(NamedInstruction::Shr),
        b"add" => Some(NamedInstruction::Add),
        b"sub" => Some(NamedInstruction::Sub),
        b"mul" => Some(NamedInstruction::Mul),
        b"imul" => Some(NamedInstruction::IMul),
        b"div" => Some(NamedInstruction::Div),
        b"idiv" => Some(NamedInstruction::IDiv),
        b"rem" => Some(NamedInstruction::Rem),
        b"cmp" => Some(NamedInstruction::Cmp),
        b"jmp" => Some(NamedInstruction::Jmp),
        b"jmpabs" => Some(NamedInstruction::JmpAbs),
        b"jz" => Some(NamedInstruction::Jz),
        b"jnz" => Some(NamedInstruction::Jnz),
        _ => None,
    }
}

#[derive(Debug, Clone)]
pub enum JmpVariant {
    Absolute,
    Relative,
}
