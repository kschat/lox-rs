use crate::{error::LoxError, value::Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum OpCode {
    Constant = 0,
    Return,
}

impl TryFrom<u8> for OpCode {
    type Error = LoxError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            value if value == OpCode::Return as u8 => Ok(OpCode::Return),
            value if value == OpCode::Constant as u8 => Ok(OpCode::Constant),
            _ => Err(LoxError::OpCodeConversionError),
        }
    }
}

impl From<OpCode> for u8 {
    fn from(value: OpCode) -> Self {
        value as u8
    }
}

pub struct Chunk {
    code: Vec<u8>,
    constants: Vec<Value>,
    pub lines: Vec<usize>,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            code: vec![],
            constants: vec![],
            lines: vec![],
        }
    }

    pub fn write(&mut self, byte: u8, line: usize) {
        self.code.push(byte);
        self.lines.push(line);
    }

    pub fn count(&self) -> usize {
        self.code.len()
    }

    pub fn get_code(&self, index: usize) -> u8 {
        self.code[index]
    }

    pub fn add_constant(&mut self, value: Value) -> usize {
        self.constants.push(value);
        self.constants.len() - 1
    }

    pub fn get_constant(&self, index: usize) -> Value {
        self.constants[index]
    }

    pub fn get_line(&self, index: usize) -> usize {
        self.lines[index]
    }
}
