use std::{fs, io, str::FromStr};

use gumdrop::Options;
use itertools::{Either, Itertools};
use vc2_assembler::{instructions::InstructionOrConstant, Assembler};

struct OutFileWrapper(String);

impl Default for OutFileWrapper {
    fn default() -> Self {
        Self(String::from("out.o"))
    }
}

impl FromStr for OutFileWrapper {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_string()))
    }
}

#[derive(Options)]
struct MyOptions {
    #[options(help = "print help message")]
    help: bool,

    #[options(
        help = "get input from <file>",
        required,
        parse(try_from_str = "read_file")
    )]
    file: String,

    #[options(help = "write output to <file>")]
    out: OutFileWrapper,

    #[options(help = "print debug output")]
    verbose: bool,
}

fn read_file(path: &str) -> io::Result<String> {
    fs::read_to_string(path)
}

fn main() {
    let MyOptions {
        file: file_contents,
        verbose,
        out: out_file,
        ..
    } = Options::parse_args_default_or_exit();

    let parser = vc2_assembler::Parser::new(file_contents.as_bytes());
    let node = parser.parse();
    let (ok, err): (Vec<InstructionOrConstant>, Vec<vc2_assembler::error::Error>) =
        node.into_iter().partition_map(|v| match v {
            Ok(v) => Either::Left(v),
            Err(v) => Either::Right(v),
        });
    if !err.is_empty() {
        let error_length = if err.len() == 1 { "error" } else { "errors" };
        println!("input has {} {error_length}:", err.len());
        for err in err {
            let contents = &file_contents[err.from.cursor..=err.to.cursor]
                .replace('\n', "\\n")
                .replace('\r', "\\r");
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
    let assembler = Assembler::new(&ok);
    let out = assembler.assemble();

    if verbose {
        println!("nodes:");
        println!("{ok:#?}");
        println!();
        println!("machine code:");
        for byte in &out {
            print!("{byte:#04X} ");
        }
        println!();
    }
    fs::write(out_file.0, out).unwrap();
}
