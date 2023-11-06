use simple_logger::SimpleLogger;
use std::io::{self, Write};

use vc2_vm::Vm;

const VM_HALT_MS: u64 = 133;
const VM_MEMORY_BYTES: usize = 0x4000;

fn vm_from_file(file_name: &str) -> io::Result<Vm<VM_MEMORY_BYTES, VM_HALT_MS>> {
    let instructions = std::fs::read(file_name)?;
    Ok(Vm::new(instructions))
}

enum WordFormat {
    Hex,
    Binary,
    Decimal,
}

fn format_word(word: u32, format: &WordFormat) -> String {
    match format {
        WordFormat::Hex => format!("{word:#010X}"),
        WordFormat::Binary => format!("{word:#34b}"),
        WordFormat::Decimal => format!("{word:#}"),
    }
}

fn main() -> Result<(), io::Error> {
    println!("[#] vc2-inspector started");
    let help_menu = include_str!("help.txt");
    let mut vm: Option<Vm<VM_MEMORY_BYTES, VM_HALT_MS>> = None;
    SimpleLogger::new()
        .without_timestamps()
        .env()
        .init()
        .unwrap();
    println!("enter commands (try `help`):");

    loop {
        print!("> ");
        io::stdout().flush().unwrap();
        let mut buffer = String::new();
        let stdin = io::stdin();
        stdin.read_line(&mut buffer)?;

        let mut buffer = buffer.split(" ").map(|v| v.trim());
        let cmd = buffer.next();
        match cmd {
            None => continue,
            Some("help") => println!("{help_menu}"),
            Some("file" | "load") => {
                let Some(file_name) = buffer.next() else {
                    println!("missing file name after `file` command");
                    continue;
                };
                match vm_from_file(file_name) {
                    Ok(new_vm) => {
                        vm = Some(new_vm);
                        println!("vm loaded from file '{file_name}'")
                    }
                    Err(err) => println!("error loading vm from file '{file_name}': {err}"),
                }
            }
            Some("step") => {
                let Some(ref mut vm) = vm else {
                    println!("vm not started, try `help`");
                    continue;
                };
                if let Err(err) = vm.run_next_instruction() {
                    println!("vm unable to step: {err}")
                }
            }
            Some("eval") => {
                let Some(ref mut vm) = vm else {
                    println!("vm not started, try `help`");
                    continue;
                };
                'eval_loop: loop {
                    if let Err(err) = vm.run_next_instruction() {
                        println!("vm unable to step: {err}");
                        break 'eval_loop;
                    }
                }
            }
            Some("registers") => {
                use vc2_vm::Register::*;
                let Some(ref vm) = vm else {
                    println!("vm not started, try `help`");
                    continue;
                };
                let format = match buffer.next() {
                    Some("hex") => WordFormat::Hex,
                    Some("binary") => WordFormat::Binary,
                    Some("decimal") => WordFormat::Decimal,
                    Some(format) => {
                        println!("unrecognized format '{format}'");
                        continue;
                    }
                    None => {
                        println!("missing format after `registers` command");
                        continue;
                    }
                };

                println!("[#] registers:");
                println!(
                    "- r0: {}",
                    format_word(vm.register_value(&GeneralPurpose0), &format)
                );
                println!(
                    "- r1: {}",
                    format_word(vm.register_value(&GeneralPurpose1), &format)
                );
                println!("- fl: {}", format_word(vm.register_value(&Flag), &format));
                println!(
                    "- pc: {}",
                    format_word(vm.register_value(&ProgramCounter), &format)
                );
            }
            Some("exit") => {
                break Ok(());
            }
            Some(cmd) => {
                println!("unrecognized cmd {cmd}");
                println!("{help_menu}");
            }
        }
    }
}
