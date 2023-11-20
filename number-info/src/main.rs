use std::io::{self, Write};

fn main() -> io::Result<()> {
    loop {
        print!("> ");
        io::stdout().flush().unwrap();
        let mut text = String::new();
        let stdin = io::stdin();
        stdin.read_line(&mut text)?;
        let text = text.replace('_', "");
        let text = text.trim();

        let value = if text.starts_with("0x") {
            i64::from_str_radix(&text[2..], 16)
        } else if text.starts_with("0b") {
            i64::from_str_radix(&text[2..], 2)
        } else {
            text.parse::<i64>()
        };
        let value = match value {
            Ok(value) => value as u32,
            Err(err) => {
                println!("invalid number '{text}': {err}");
                continue;
            }
        };

        let unsigned_len = value.to_string().len();
        let signed_len = {
            let value = value as i32;
            if value < 0 {
                value.to_string().len() - 1
            } else {
                value.to_string().len()
            }
        };
        let hex_len = format!("{value:X}").len();
        let binary_len = format!("{value:b}").len();

        println!("value:");
        println!("   hex:      {value:#X} ({hex_len} digits)");
        println!("   binary:   {value:#b} ({binary_len} digits)");
        println!("   unsigned: {value} ({unsigned_len} digits)");
        println!("   signed:   {} ({signed_len} digits)", value as i32);
    }
}
