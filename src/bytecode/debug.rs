use crate::chunk::{Chunk, OpCode};
use crate::error::Result;

pub struct Disassembler<'a> {
    chunk: &'a Chunk,
}

impl<'a> Disassembler<'a> {
    pub fn new(chunk: &'a Chunk) -> Self {
        Self { chunk }
    }

    pub fn process_chunk(&self, name: &str) -> Result<()> {
        println!("== {} ==", name);

        let mut offset = 0;

        loop {
            if offset >= self.chunk.count() {
                return Ok(());
            }

            offset = self.process_instruction(offset)?;
        }
    }

    pub fn process_instruction(&self, offset: usize) -> Result<usize> {
        print!("{:04} ", offset);
        print!("{:>4} ", self.get_line_label(offset));

        let instruction = self.chunk.get_code(offset);
        Ok(match instruction.try_into() {
            Ok(code @ OpCode::Return) => self.simple_instruction(code.as_ref(), offset),
            Ok(code @ OpCode::Negate) => self.simple_instruction(code.as_ref(), offset),
            Ok(code @ OpCode::Add) => self.simple_instruction(code.as_ref(), offset),
            Ok(code @ OpCode::Subtract) => self.simple_instruction(code.as_ref(), offset),
            Ok(code @ OpCode::Multiply) => self.simple_instruction(code.as_ref(), offset),
            Ok(code @ OpCode::Divide) => self.simple_instruction(code.as_ref(), offset),
            Ok(code @ OpCode::Constant) => self.constant_instruction(code.as_ref(), offset),
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

    fn constant_instruction(&self, name: &str, offset: usize) -> usize {
        let constant_index = self.chunk.get_code(offset + 1) as usize;
        let constant = self.chunk.get_constant(constant_index);
        println!("{: <16} {:4} '{}'", name, constant_index, constant);
        offset + 2
    }

    fn get_line_label(&self, offset: usize) -> String {
        let line = self.chunk.get_line(offset);
        if offset > 0 && line == self.chunk.get_line(offset - 1) {
            return "|".to_string();
        }

        line.to_string()
    }
}
