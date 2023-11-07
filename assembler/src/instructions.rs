#[derive(Debug, PartialEq)]
pub enum Target {
    Register(Register),
    RegisterAddress(Register),
    Immediate(Immediate),
    ImmediateAddress(Immediate),
}

pub type Immediate = u32;

#[derive(Debug, PartialEq)]
pub enum Register {
    GeneralPurpose0,
    GeneralPurpose1,
    Flag,
    ProgramCounter,
}

#[derive(Debug)]
pub enum Instruction {
    Nop,
    Htl,
    Mov(Target, Target),
    Not(Register),
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
pub enum JmpVariant {
    Absolute,
    Relative,
}
