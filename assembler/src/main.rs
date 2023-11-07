use std::{fs, io};

use gumdrop::Options;

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
    parser.parse();
}
