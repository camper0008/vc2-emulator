use simple_logger::SimpleLogger;
use std::io::{self, Write};

use vc2_vm::Vm;

const VM_HALT_MS: u64 = 133;
const VM_MEMORY_BYTES: usize = 0x4000;

fn vm_from_file(file_name: &str) -> io::Result<Vm<VM_MEMORY_BYTES, VM_HALT_MS>> {
    let instructions = std::fs::read(file_name)?;
    Ok(Vm::new(instructions))
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
                let file_name = buffer.next();
                let Some(file_name) = file_name else {
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
