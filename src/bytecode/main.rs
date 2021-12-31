use chunk::{Chunk, OpCode};
use debug::Disassembler;

use crate::error::Result;

mod chunk;
mod debug;
mod error;
mod value;

fn main() -> Result<()> {
    let mut chunk = Chunk::new();

    let constant = chunk.add_constant(1.2);
    chunk.write(OpCode::Constant.into(), 123);
    chunk.write(constant as u8, 123);

    chunk.write(OpCode::Return.into(), 123);

    Disassembler.process_chunk(&chunk, "test chunk")?;

    Ok(())
}
