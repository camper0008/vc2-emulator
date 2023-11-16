use simple_logger::SimpleLogger;
use std::io::{self, Write};
use utils::parse_integer;

use vc2_vm::Vm;
mod utils;

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

#[derive(PartialEq)]
enum CmdResult {
    Continue,
    Exit,
}

fn word_format(cmd: &str, word: Option<&str>) -> Option<WordFormat> {
    match word {
        Some("hex") => Some(WordFormat::Hex),
        Some("bin" | "binary") => Some(WordFormat::Binary),
        Some("dec" | "decimal") => Some(WordFormat::Decimal),
        Some(format) => {
            println!("unrecognized output format '{format}'");
            None
        }
        None => {
            println!("missing output format after `{cmd}` command");
            None
        }
    }
}

fn execute_cmd(
    vm: &mut Option<Vm<VM_MEMORY_BYTES, VM_HALT_MS>>,
    buffer: &mut dyn Iterator<Item = &str>,
) -> CmdResult {
    let help_menu = include_str!("help.txt");

    let cmd = buffer.next();
    match cmd {
        Some("help") => println!("{help_menu}"),
        Some(cmd @ ("file" | "load")) => {
            let Some(file_name) = buffer.next() else {
                    println!("missing file name after `{cmd}` command");
                    return CmdResult::Continue;
                };
            match vm_from_file(file_name) {
                Ok(new_vm) => {
                    *vm = Some(new_vm);
                    println!("vm loaded from file '{file_name}'")
                }
                Err(err) => println!("error loading vm from file '{file_name}': {err}"),
            }
        }
        Some("step") => {
            let Some(ref mut vm) = vm else {
                    println!("vm not started, try `help`");
                   return CmdResult::Continue;
                };

            let amount = buffer.next().map(parse_integer).unwrap_or(Ok(1));
            let Ok(amount): Result<usize, _> = amount else {
                    println!("amount '{}' is not a usize", amount.unwrap_err());
                   return CmdResult::Continue;
                };

            (0..amount).for_each(|_| {
                if let Err(err) = vm.run_next_instruction() {
                    println!("vm unable to step: {err}")
                }
            });
        }
        Some("inline") => {
            let mut bytes = Vec::new();

            loop {
                let Some(byte) = buffer.next() else {
                    break;
                };
                if byte == "&&" {
                    println!("&& is not supported with the inline command");
                    break;
                }

                let Some(byte) = buffer.next() else {
                    unreachable!();
                };

                bytes.push(match parse_integer::<u8>(byte) {
                    Ok(v) => v,
                    Err(err) => {
                        println!("invalid byte {byte}: '{err}'");
                        break;
                    }
                });
            }
            *vm = Some(Vm::new(bytes));
            println!("vm loaded from bytes")
        }
        Some(cmd @ "repeat") => {
            let Some(amount) = buffer.next() else {
                    println!("missing amount after `{cmd}` command");
                   return CmdResult::Continue;
                };
            let Ok(amount): Result<usize, _> = amount.parse() else {
                    println!("steps '{}' is not a usize", amount);
                   return CmdResult::Continue;
                };

            let buffer = buffer.collect::<Vec<_>>();
            for _ in 0..amount {
                let mut buffer = buffer.clone().into_iter();
                let result = execute_cmd(vm, &mut buffer);
                if CmdResult::Exit == result {
                    return CmdResult::Exit;
                }
            }
        }
        Some("eval") => {
            let Some(ref mut vm) = vm else {
                    println!("vm not started, try `help`");
                   return CmdResult::Continue;
                };
            'eval_loop: loop {
                if let Err(err) = vm.run_next_instruction() {
                    println!("vm unable to step: {err}");
                    break 'eval_loop;
                }
            }
        }
        Some(cmd @ "registers") => {
            use vc2_vm::Register::*;
            let Some(vm) = vm else {
                    println!("vm not started, try `help`");
                    return CmdResult::Continue;
                };
            let Some(format) = word_format(cmd, buffer.next()) else {
                return CmdResult::Continue;
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
        Some(cmd @ "memory") => {
            let Some(vm) = vm else {
                    println!("vm not started, try `help`");
                    return CmdResult::Continue;
                };
            let Some(format) = word_format(cmd, buffer.next()) else {
                    return CmdResult::Continue;
                };

            let start = buffer.next().map(|v| parse_integer(v).ok()).flatten();
            let Some(start) = start else {
                println!("invalid memory start after `{cmd}`");
                return CmdResult::Continue;
            };
            let stop = buffer.next().map(|v| parse_integer(v).ok()).flatten();
            let Some(stop) = stop else {
                println!("invalid memory stop after `{cmd}`");
                return CmdResult::Continue;
            };

            if start >= stop {
                println!("start {start} cannot be >= stop {stop}; range is exclusive");
                return CmdResult::Continue;
            }

            println!("[#] memory:");
            print!("[");
            for i in start..stop {
                let value = match vm.memory_value(&i) {
                    Ok(v) => v,
                    Err(err) => {
                        println!("]");

                        println!(
                            "unable to access memory at {}:\n  '{err}'",
                            format_word(i, &format)
                        );
                        return CmdResult::Continue;
                    }
                };
                print!("{}", format_word(value, &format));
                if i < stop - 1 {
                    print!(", ");
                }
            }
            println!("]");
        }

        Some("exit") => {
            return CmdResult::Exit;
        }
        Some(cmd) => {
            println!("unrecognized cmd {cmd}");
            println!("{help_menu}");
        }
        None => {}
    };
    match buffer.next() {
        Some("&&") => execute_cmd(vm, buffer),
        Some(cmd) => {
            println!("unrecognized trailing input '{cmd}'");
            CmdResult::Continue
        }
        None => CmdResult::Continue,
    }
}

fn main() -> Result<(), io::Error> {
    println!("[#] vc2-inspector started");
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

        let mut buffer = buffer.split(' ').map(|v| v.trim());
        if execute_cmd(&mut vm, &mut buffer) == CmdResult::Exit {
            break Ok(());
        };
    }
}
