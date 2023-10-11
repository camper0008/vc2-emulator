use std::io;

use vc2_vm::Vm;

fn main() -> Result<(), io::Error> {
    const VM_HALT_MS: u64 = 133;
    const VM_MEMORY_BYTES: usize = 2_000_000;
    let help_menu = include_str!("help.txt");
    let _vm: Option<Vm<VM_MEMORY_BYTES, VM_HALT_MS>> = None;
    loop {
        let mut buffer = String::new();
        let stdin = io::stdin();
        stdin.read_line(&mut buffer)?;

        let mut buffer = buffer.split(" ");
        let cmd = buffer.next();
        match cmd {
            None => continue,
            Some("help") => println!("{help_menu}"),
            Some(cmd) => {
                println!("unrecognized cmd {cmd}");
                println!("{help_menu}");
            }
        }
    }
}
