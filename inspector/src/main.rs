use gumdrop::Options;
use log::LevelFilter;
use simple_logger::SimpleLogger;
use std::{
    io::{self, Write},
    sync::{Arc, Mutex},
    time::Instant,
};
use utils::parse_integer;

use vc2_vm::Vm;

mod utils;

const VM_MEMORY_BYTES: usize = 0x30000;

#[cfg(feature = "peripherals")]
mod peripherals;

fn vm_from_file(file_name: &str) -> io::Result<Vm> {
    let instructions = std::fs::read(file_name)?;
    Ok(Vm::new(instructions, VM_MEMORY_BYTES))
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

fn initialize_vm(vm: &mut Vm) -> Result<(), String> {
    #[cfg(feature = "peripherals")]
    {
        vm.set_memory_value(&peripherals::SCREEN_ENABLED_LOCATION, 1)?;
        vm.set_memory_value(
            &peripherals::SCREEN_VRAM_ADDRESS_LOCATION,
            peripherals::SCREEN_VRAM_ADDRESS,
        )?;
        vm.set_memory_value(
            &peripherals::SCREEN_WIDTH_LOCATION,
            peripherals::SCREEN_WIDTH,
        )?;
        vm.set_memory_value(
            &peripherals::SCREEN_HEIGHT_LOCATION,
            peripherals::SCREEN_HEIGHT,
        )?;
    }

    Ok(())
}

fn execute_cmd(
    vm: &mut Arc<Mutex<Option<Vm>>>,
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
                Ok(mut new_vm) => {
                    let mut vm = vm.lock().unwrap();
                    initialize_vm(&mut new_vm).unwrap();
                    *vm = Some(new_vm);
                    drop(vm);
                    println!("vm loaded from file '{file_name}'")
                }
                Err(err) => println!("error loading vm from file '{file_name}': {err}"),
            }
        }
        Some("step") => {
            let mut vm = vm.lock().unwrap();
            let Some(ref mut vm) = *vm else {
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

                bytes.push(match parse_integer::<u8>(byte) {
                    Ok(v) => v,
                    Err(err) => {
                        println!("invalid byte {byte}: '{err}'");
                        break;
                    }
                });
            }
            let mut vm = vm.lock().unwrap();
            let mut new_vm = Vm::new(bytes, VM_MEMORY_BYTES);
            initialize_vm(&mut new_vm).unwrap();
            *vm = Some(new_vm);
            println!("vm loaded from bytes");
            drop(vm);
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
            let mut steps = 0;
            let now = Instant::now();
            'eval_loop: loop {
                let mut vm_ref = vm.lock().unwrap();
                let Some(ref mut vm) = *vm_ref else {
                    println!("vm not started, try `help`");
                    return CmdResult::Continue;
                };
                if let Err(err) = vm.run_next_instruction() {
                    println!("vm unable to step: {err}");
                    break 'eval_loop;
                }
                drop(vm_ref);
                steps += 1;
            }
            let now = Instant::now() - now;
            println!(
                "evaluated program in {}ms and {steps} steps",
                now.as_millis()
            );
        }
        Some(cmd @ "registers") => {
            use vc2_vm::Register::*;
            let vm = vm.lock().unwrap();
            let Some(ref vm) = *vm else {
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
            let vm = vm.lock().unwrap();
            let Some(ref vm) = *vm else {
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

#[derive(Options)]
struct MyOptions {
    #[options(help = "print help message")]
    help: bool,

    #[options(help = "log level (off, debug, warn, error)", default = "off")]
    log_level: LevelFilter,
}

fn main() -> Result<(), io::Error> {
    let MyOptions { log_level, .. } = Options::parse_args_default_or_exit();
    println!("[#] vc2-inspector started");
    let mut vm: Arc<Mutex<Option<Vm>>> = Arc::new(Mutex::new(None));
    SimpleLogger::new()
        .without_timestamps()
        .with_level(log_level)
        .init()
        .unwrap();
    println!("enter commands (try `help`):");

    #[cfg(feature = "peripherals")]
    peripherals::window(vm.clone());

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
