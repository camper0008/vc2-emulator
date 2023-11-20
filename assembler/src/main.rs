use std::{fs, io, str::FromStr};

use gumdrop::Options;
use itertools::{Either, Itertools};
use log::LevelFilter;
use simple_logger::SimpleLogger;
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

    #[options(help = "log level (off, debug, info, warn, error)", default = "info")]
    log_level: LevelFilter,
}

fn read_file(path: &str) -> io::Result<String> {
    fs::read_to_string(path)
}

fn main() {
    let MyOptions {
        file: file_contents,
        out: out_file,
        log_level,
        ..
    } = Options::parse_args_default_or_exit();

    SimpleLogger::new()
        .without_timestamps()
        .with_level(log_level)
        .init()
        .unwrap();

    let parser = vc2_assembler::Parser::new(file_contents.as_bytes());
    let node = parser.parse();
    let (ok, err): (Vec<InstructionOrConstant>, Vec<vc2_assembler::error::Error>) =
        node.into_iter().partition_map(|v| match v {
            Ok(v) => Either::Left(v),
            Err(v) => Either::Right(v),
        });
    if !err.is_empty() {
        let error_length = if err.len() == 1 { "error" } else { "errors" };
        log::error!("input has {} {error_length}:", err.len());
        for err in err {
            let contents = &file_contents[err.from.cursor..=err.to.cursor]
                .replace('\n', "\\n")
                .replace('\r', "\\r");
            if err.from.line == err.to.line && err.from.character == err.to.character {
                log::error!(
                    "@ ({}:{}) {} '{contents}'",
                    err.from.line,
                    err.from.character,
                    err.message,
                );
            } else {
                log::error!(
                    "@ {}:{}-{}:{} {} '{contents}'",
                    err.from.line,
                    err.from.character,
                    err.to.line,
                    err.to.character,
                    err.message,
                );
            }
        }
        std::process::exit(1);
    }
    let assembler = Assembler::new(&ok);
    let out = assembler.assemble();

    log::debug!("nodes:");
    log::debug!("{ok:#?}");
    log::debug!("machine code:");
    log::debug!(
        "{:?}",
        out.iter()
            .map(|byte| format!("{byte:#04X}"))
            .reduce(|acc, v| acc + ", " + &v)
    );

    fs::write(out_file.0, out).unwrap();
}
