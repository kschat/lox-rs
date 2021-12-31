use crate::chunk::{Chunk, OpCode};
use crate::error::Result;

pub struct Disassembler;

impl Disassembler {
    pub fn process_chunk(&self, chunk: &Chunk, name: &str) -> Result<()> {
        println!("== {} ==", name);

        let mut offset = 0;

        loop {
            if offset >= chunk.count() {
                return Ok(());
            }

            offset = self.process_instruction(chunk, offset)?;
        }
    }

    fn process_instruction(&self, chunk: &Chunk, offset: usize) -> Result<usize> {
        print!("{:04} ", offset);
        print!("{:>4} ", self.get_line_label(chunk, offset));

        let instruction = chunk.get_code(offset);
        Ok(match instruction.try_into() {
            Ok(OpCode::Return) => self.simple_instruction("OP_RETURN", offset),
            Ok(OpCode::Constant) => self.constant_instruction("OP_CONSTANT", chunk, offset),
            Err(_) => {
                println!("Unknown opcode {}", instruction);
                offset + 1
            }
        })
    }

    fn simple_instruction(&self, name: &str, offset: usize) -> usize {
        println!("{}", name);
        offset + 1
    }

    fn constant_instruction(&self, name: &str, chunk: &Chunk, offset: usize) -> usize {
        let constant_index = chunk.get_code(offset + 1) as usize;
        let constant = chunk.get_constant(constant_index);
        println!("{: <16} {:4} '{}'", name, constant_index, constant);
        offset + 2
    }

    fn get_line_label(&self, chunk: &Chunk, offset: usize) -> String {
        let line = chunk.get_line(offset);
        if offset > 0 && line == chunk.get_line(offset - 1) {
            return "|".to_string();
        }

        line.to_string()
    }
}
