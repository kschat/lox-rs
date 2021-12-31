use chunk::{Chunk, OpCode};
use debug::Disassembler;
use structopt::StructOpt;
use vm::Vm;

use crate::error::Result;

mod chunk;
mod debug;
mod error;
mod value;
mod vm;

#[derive(StructOpt, Debug)]
#[structopt(name = "blox")]
struct CommandOptions {
    #[structopt(short, long)]
    debug: bool,
}

fn main() -> Result<()> {
    let command_options = CommandOptions::from_args();

    let mut chunk = Chunk::new();

    let line = 123;
    let constant = chunk.add_constant(1.2);
    chunk.write(OpCode::Constant.into(), line);
    chunk.write(constant as u8, line);

    let constant = chunk.add_constant(3.4);
    chunk.write(OpCode::Constant.into(), line);
    chunk.write(constant as u8, line);

    chunk.write(OpCode::Add.into(), line);

    let constant = chunk.add_constant(5.6);
    chunk.write(OpCode::Constant.into(), line);
    chunk.write(constant as u8, line);

    chunk.write(OpCode::Divide.into(), line);
    chunk.write(OpCode::Negate.into(), line);

    chunk.write(OpCode::Return.into(), line);

    Disassembler::new(&chunk).process_chunk("test chunk")?;
    Vm::interpret(
        chunk,
        vm::VmConfig {
            debug: command_options.debug,
        },
    )?;

    Ok(())
}
