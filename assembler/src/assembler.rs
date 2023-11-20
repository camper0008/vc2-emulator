use std::collections::HashMap;

use crate::instructions::{
    Instruction, InstructionOrConstant, JmpVariant, PreprocessorCommand, Register, Target,
};

enum PreprocessorConstant {
    Label(u32),
    Define(u32),
}

pub struct Assembler<'a> {
    cursor: usize,
    inner: &'a [InstructionOrConstant],
    constants: HashMap<String, PreprocessorConstant>,
    current_label: Option<String>,
    instructions: Vec<IntermediaryOutput>,
}

#[derive(Debug, PartialEq)]
enum IntermediaryOutput {
    Byte(u8),
    ConstantReference { name: String, position: usize },
    ConstantPadding,
}

impl<'a> Assembler<'a> {
    #[must_use]
    pub fn new(inner: &'a [InstructionOrConstant]) -> Self {
        Self {
            current_label: None,
            cursor: 0,
            instructions: Vec::new(),
            inner,
            constants: HashMap::new(),
        }
    }
    fn selector_from_target(target: &Target) -> u8 {
        match target {
            Target::Register(_) => 0b00,
            Target::Immediate(_) | Target::Constant(_) | Target::SubConstant(_) => 0b01,
            Target::RegisterAddress(_) => 0b10,
            Target::ImmediateAddress(_)
            | Target::ConstantAddress(_)
            | Target::SubConstantAddress(_) => 0b11,
        }
    }
    fn register_byte(register: &Register) -> u8 {
        match register {
            Register::GeneralPurpose0 => 0b00,
            Register::GeneralPurpose1 => 0b01,
            Register::Flag => 0b10,
            Register::ProgramCounter => 0b11,
        }
    }

    fn label_key(&self, sub_label: &'a str) -> Option<String> {
        self.current_label
            .as_ref()
            .map(|parent| format!("{parent}@{sub_label}"))
    }
    fn instruction_byte(instruction: &Instruction) -> u8 {
        match instruction {
            Instruction::Nop => 0x00,
            Instruction::Hlt => 0x01,
            Instruction::Mov(_, _) => 0x02,
            Instruction::Or(_, _) => 0x03,
            Instruction::And(_, _) => 0x04,
            Instruction::Xor(_, _) => 0x05,
            Instruction::Not(_) => 0x06,
            Instruction::Shl(_, _) => 0x07,
            Instruction::Shr(_, _) => 0x08,
            Instruction::Add(_, _) => 0x09,
            Instruction::Sub(_, _) => 0x0a,
            Instruction::Mul(_, _) => 0x0b,
            Instruction::IMul(_, _) => 0x0c,
            Instruction::Div(_, _) => 0x0d,
            Instruction::IDiv(_, _) => 0x0e,
            Instruction::Rem(_, _) => 0x0f,
            Instruction::Cmp(_, _) => 0x10,
            Instruction::Jmp(_, _) => 0x11,
            Instruction::Jz(_, _) => 0x12,
            Instruction::Jnz(_, _) => 0x13,
        }
    }
    fn push_immediate(instructions: &mut Vec<IntermediaryOutput>, immediate: u32) {
        for byte in immediate.to_be_bytes() {
            instructions.push(IntermediaryOutput::Byte(byte));
        }
    }
    fn push_constant_reference(
        instructions: &mut Vec<IntermediaryOutput>,
        name: String,
        position: usize,
    ) {
        instructions.push(IntermediaryOutput::ConstantReference { name, position });
        instructions.push(IntermediaryOutput::ConstantPadding);
        instructions.push(IntermediaryOutput::ConstantPadding);
        instructions.push(IntermediaryOutput::ConstantPadding);
    }
    fn assemble_next(&mut self) -> bool {
        use IntermediaryOutput::Byte;
        let current = self.current();
        match current {
            InstructionOrConstant::Instruction(instruction) => {
                self.instructions
                    .push(Byte(Self::instruction_byte(&instruction)));
                match instruction {
                    Instruction::Nop | Instruction::Hlt => self.step(),
                    Instruction::Not(target) => {
                        let selector = Self::selector_from_target(&target);

                        match &target {
                            Target::Register(register) | Target::RegisterAddress(register) => {
                                let register = Self::register_byte(register);
                                self.instructions.push(Byte(selector << 6 | register << 2));
                            }
                            Target::ImmediateAddress(immediate) => {
                                self.instructions.push(Byte(selector << 6));
                                Self::push_immediate(&mut self.instructions, *immediate);
                            }
                            Target::ConstantAddress(label) => {
                                let instruction_position = self.instructions.len() - 1;
                                self.instructions.push(Byte(selector << 6));
                                Self::push_constant_reference(
                                    &mut self.instructions,
                                    label.to_owned(),
                                    instruction_position,
                                )
                            }
                            Target::SubConstantAddress(label) => {
                                let instruction_position = self.instructions.len() - 1;
                                self.instructions.push(Byte(selector << 6));

                                let Some(label) = self.label_key(&label) else {
                                    todo!("reached sub constant without label")
                                };

                                Self::push_constant_reference(
                                    &mut self.instructions,
                                    label,
                                    instruction_position,
                                );
                            }
                            Target::Immediate(_) | Target::Constant(_) | Target::SubConstant(_) => {
                                unreachable!()
                            }
                        }
                        self.step();
                    }
                    Instruction::Jz(dest, src)
                    | Instruction::Jnz(dest, src)
                    | Instruction::Mov(dest, src)
                    | Instruction::Or(dest, src)
                    | Instruction::And(dest, src)
                    | Instruction::Xor(dest, src)
                    | Instruction::Shl(dest, src)
                    | Instruction::Shr(dest, src)
                    | Instruction::Add(dest, src)
                    | Instruction::Sub(dest, src)
                    | Instruction::Mul(dest, src)
                    | Instruction::IMul(dest, src)
                    | Instruction::Div(dest, src)
                    | Instruction::IDiv(dest, src)
                    | Instruction::Rem(dest, src)
                    | Instruction::Cmp(dest, src) => {
                        let dest_selector = Self::selector_from_target(&dest);
                        let src_selector = Self::selector_from_target(&src);
                        let instruction_position = self.instructions.len() - 1;
                        let mut to_add = Vec::new();
                        to_add.push(Byte(dest_selector << 6 | src_selector << 4));

                        match dest {
                            Target::Register(register) | Target::RegisterAddress(register) => {
                                match to_add.first_mut() {
                                    Some(Byte(v)) => *v |= Self::register_byte(&register) << 2,
                                    _ => unreachable!(),
                                }
                            }
                            Target::Immediate(immediate) | Target::ImmediateAddress(immediate) => {
                                Self::push_immediate(&mut to_add, immediate);
                            }
                            Target::Constant(label) | Target::ConstantAddress(label) => {
                                Self::push_constant_reference(
                                    &mut to_add,
                                    label,
                                    instruction_position,
                                );
                            }
                            Target::SubConstant(label) | Target::SubConstantAddress(label) => {
                                let Some(label) = self.label_key(&label) else {
                                    todo!("reached sub label without label")
                                };
                                Self::push_constant_reference(
                                    &mut to_add,
                                    label,
                                    instruction_position,
                                );
                            }
                        }
                        match src {
                            Target::Register(register) | Target::RegisterAddress(register) => {
                                match to_add.first_mut() {
                                    Some(Byte(v)) => *v |= Self::register_byte(&register),
                                    _ => unreachable!(),
                                }
                            }
                            Target::Immediate(immediate) | Target::ImmediateAddress(immediate) => {
                                Self::push_immediate(&mut to_add, immediate);
                            }
                            Target::Constant(label) | Target::ConstantAddress(label) => {
                                Self::push_constant_reference(
                                    &mut to_add,
                                    label,
                                    instruction_position,
                                );
                            }
                            Target::SubConstant(label) | Target::SubConstantAddress(label) => {
                                let Some(label) = self.label_key(&label) else {
                                    todo!("reached sub label without label")
                                };
                                Self::push_constant_reference(
                                    &mut to_add,
                                    label,
                                    instruction_position,
                                );
                            }
                        }
                        self.instructions.append(&mut to_add);
                        self.step();
                    }
                    Instruction::Jmp(dest, variant) => {
                        let dest_selector = Self::selector_from_target(&dest);
                        let mut to_add = Vec::new();
                        let variant = match variant {
                            JmpVariant::Absolute => 1,
                            JmpVariant::Relative => 0,
                        };
                        to_add.push(Byte(dest_selector << 6 | variant));
                        let instruction_position = self.instructions.len() - 1;

                        match dest {
                            Target::Register(register) | Target::RegisterAddress(register) => {
                                match to_add.first_mut() {
                                    Some(Byte(v)) => *v |= Self::register_byte(&register) << 2,
                                    _ => unreachable!(),
                                }
                            }
                            Target::Immediate(immediate) | Target::ImmediateAddress(immediate) => {
                                Self::push_immediate(&mut to_add, immediate);
                            }
                            Target::Constant(label) | Target::ConstantAddress(label) => {
                                Self::push_constant_reference(
                                    &mut to_add,
                                    label,
                                    instruction_position,
                                );
                            }
                            Target::SubConstant(label) | Target::SubConstantAddress(label) => {
                                let Some(label) = self.label_key(&label) else {
                                    todo!("reached sub label without label")
                                };
                                Self::push_constant_reference(
                                    &mut to_add,
                                    label,
                                    instruction_position,
                                );
                            }
                        }
                        self.instructions.append(&mut to_add);
                        self.step();
                    }
                }
            }
            InstructionOrConstant::PreprocessorCommand(command) => match command {
                PreprocessorCommand::Offset(offset) => {
                    for _ in 0..offset {
                        self.instructions.push(IntermediaryOutput::Byte(0x0));
                    }
                    self.step();
                }
                PreprocessorCommand::Define(name, value) => {
                    let name_key = name.clone();
                    let existing_label = self
                        .constants
                        .insert(name_key, PreprocessorConstant::Define(value));
                    match existing_label {
                        Some(PreprocessorConstant::Define(v)) if v == value => {}
                        Some(PreprocessorConstant::Label(v)) => {
                            todo!("constant '{name}' is also the name of a label pointing to {v}")
                        }
                        Some(PreprocessorConstant::Define(v)) => {
                            todo!("constants must be unique, '{name}' already exists with a value of {v}")
                        }
                        None => {}
                    }
                    self.step();
                }
                PreprocessorCommand::DefineSub(label, value) => {
                    let Some(label) = self.label_key(&label) else {
                        todo!("sublabel without label");
                    };

                    let existing_constant = self
                        .constants
                        .insert(label.clone(), PreprocessorConstant::Define(value));
                    match existing_constant {
                        Some(PreprocessorConstant::Define(v)) if v == value => {}
                        Some(PreprocessorConstant::Label(v)) => {
                            todo!("constant '{label}' is also the name of a label pointing to {v}")
                        }
                        Some(PreprocessorConstant::Define(v)) => {
                            todo!("constants must be unique, '{label}' already exists with a value of {v}")
                        }
                        None => {}
                    }
                    self.step();
                }
            },
            InstructionOrConstant::Label(label) => {
                let position = self.instructions.len();
                let position = position.try_into().unwrap();
                let existing_label = self
                    .constants
                    .insert(label.to_string(), PreprocessorConstant::Label(position));
                match existing_label {
                    Some(PreprocessorConstant::Label(v)) if v == position => {}
                    Some(PreprocessorConstant::Define(v)) => {
                        todo!("label '{label}' is also the name of a constant with value {v}")
                    }
                    None => {}
                    Some(PreprocessorConstant::Label(v)) => {
                        todo!("labels must be unique, label '{label}' exists at {v} and {position}")
                    }
                }
                self.current_label = Some(label.to_string());
                self.step();
            }
            InstructionOrConstant::SubLabel(label) => {
                let position = self.instructions.len();
                let position = position.try_into().unwrap();
                let Some(label_key) = self.label_key(&label) else {
                    todo!("sublabel without label");
                };
                let existing_label = self
                    .constants
                    .insert(label_key, PreprocessorConstant::Label(position));
                match existing_label {
                    Some(PreprocessorConstant::Label(v)) if v == position => {}
                    Some(PreprocessorConstant::Define(v)) => {
                        todo!("label '{label}' is also the name of a constant with value {v}")
                    }
                    None => {}
                    Some(PreprocessorConstant::Label(v)) => {
                        todo!("labels must be unique, label '{label}' exists at {v} and {position}")
                    }
                }
                self.step();
            }
            InstructionOrConstant::EOF => return true,
        }
        false
    }
    #[must_use]
    pub fn assemble(mut self) -> Vec<u8> {
        loop {
            if self.assemble_next() {
                break;
            }
        }
        let mut out = Vec::new();
        let mut instructions = self.instructions.iter();
        use IntermediaryOutput::{Byte, ConstantPadding, ConstantReference};

        loop {
            let Some(next) = instructions.next() else {
                break;
            };
            match next {
                Byte(v) => out.push(*v),
                ConstantReference { name, position } => {
                    let (label, is_abs) = match name.strip_prefix("abs_") {
                        Some(label) => (label, true),
                        None => (name.as_str(), false),
                    };
                    let Some(value) = self.constants.get(label) else {
                        todo!("error: unrecognized constant '{label}' with value {position}");
                    };
                    let value = match value {
                        PreprocessorConstant::Define(value) => *value,
                        PreprocessorConstant::Label(value) if is_abs => *value,
                        PreprocessorConstant::Label(value) => {
                            (*value as i32 - *position as i32) as u32
                        }
                    };
                    for _ in 0..3 {
                        let Some(next) = instructions.next() else {
                            unreachable!("a reference should always be followed by 3 paddings");
                        };
                        assert_eq!(
                            next, &ConstantPadding,
                            "a reference should always be followed by 3 paddings"
                        );
                    }
                    for i in value.to_be_bytes() {
                        out.push(i);
                    }
                }
                ConstantPadding => {
                    unreachable!("should consume any label padding")
                }
            }
        }
        assert_eq!(
            out.len(),
            self.instructions.len(),
            "instructions should have same length as output so positions match"
        );
        out
    }
    pub fn step(&mut self) {
        self.cursor += 1;
    }
    #[must_use]
    pub fn current(&self) -> InstructionOrConstant {
        self.inner[self.cursor].clone()
    }
}
