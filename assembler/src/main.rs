use std::{fs, io};

use gumdrop::Options;
use itertools::{Either, Itertools};
use vc2_assembler::{instructions::InstructionOrLabel, Assembler};

#[derive(Options)]
struct MyOptions {
    #[options(help = "print help message")]
    help: bool,

    #[options(
        help = "path to file to convert",
        required,
        parse(try_from_str = "read_file")
    )]
    file: String,
}

fn read_file(path: &str) -> io::Result<String> {
    fs::read_to_string(path)
}

fn main() {
    let MyOptions {
        file: file_contents,
        ..
    } = Options::parse_args_default_or_exit();

    let parser = vc2_assembler::Parser::new(file_contents.as_bytes());
    let node = parser.parse();
    let (ok, err): (Vec<InstructionOrLabel>, Vec<vc2_assembler::error::Error>) =
        node.into_iter().partition_map(|v| match v {
            Ok(v) => Either::Left(v),
            Err(v) => Either::Right(v),
        });
    if err.len() != 0 {
        let error_length = if err.len() == 1 { "error" } else { "errors" };
        println!("input has {} {error_length}:", err.len());
        for err in err {
            let contents = &file_contents[err.from.cursor..=err.to.cursor]
                .replace("\n", "\\n")
                .replace("\r", "\\r");
            if err.from.line == err.to.line && err.from.character == err.to.character {
                println!(
                    "@ ({}:{}) {} '{contents}'",
                    err.from.line, err.from.character, err.message,
                );
            } else {
                println!(
                    "@ {}:{}-{}:{} {} '{contents}'",
                    err.from.line, err.from.character, err.to.line, err.to.character, err.message,
                );
            }
        }
        std::process::exit(1);
    }
    println!("{ok:#?}");
    let assembler = Assembler::new(&ok);
    let out = assembler.assemble();
    fs::write("out.o", out).unwrap();
}
