use std::fmt::Display;

use crate::{
    chunk::{Chunk, OpCode},
    debug::Disassembler,
    error::Result,
    value::Value,
};

const STACK_MAX: usize = 256;

#[derive(Debug)]
struct Stack {
    values: [Value; STACK_MAX],
    /// Tracks the next available location in the stack
    top: usize,
}

impl Stack {
    pub fn new() -> Self {
        Self {
            values: [0.0; STACK_MAX],
            top: 0,
        }
    }

    pub fn push(&mut self, value: Value) {
        self.values[self.top] = value;
        self.top += 1;
    }

    pub fn pop(&mut self) -> Value {
        self.top -= 1;
        self.values[self.top]
    }
}

impl Display for Stack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut i = 0;
        while i < self.top {
            write!(f, "[ {} ]", self.values[i])?;
            i += 1;
        }

        Ok(())
    }
}

pub struct VmConfig {
    pub debug: bool,
}

pub struct Vm {
    config: VmConfig,
    code: Chunk,
    /// Instruction Pointer: tracks the _next_ instruction to be executed
    ip: usize,
    stack: Stack,
}

impl Vm {
    pub fn interpret(code: Chunk, config: VmConfig) -> Result<()> {
        let mut vm = Vm {
            config,
            code,
            ip: 0,
            stack: Stack::new(),
        };

        vm.run()
    }

    fn run(&mut self) -> Result<()> {
        loop {
            if self.config.debug {
                println!("          {}", self.stack);
                Disassembler::new(&self.code).process_instruction(self.ip)?;
            }

            let instruction = self.read_byte();
            match instruction.try_into()? {
                OpCode::Return => {
                    println!("{}", self.stack.pop());
                    return Ok(());
                }
                OpCode::Negate => {
                    let value = -self.stack.pop();
                    self.stack.push(value);
                }
                OpCode::Add => self.binary_op(|a, b| a + b),
                OpCode::Subtract => self.binary_op(|a, b| a - b),
                OpCode::Multiply => self.binary_op(|a, b| a * b),
                OpCode::Divide => self.binary_op(|a, b| a / b),
                OpCode::Constant => {
                    let index = self.read_byte() as usize;
                    let constant = self.code.get_constant(index);
                    self.stack.push(constant);
                }
            }
        }
    }

    fn read_byte(&mut self) -> u8 {
        let byte = self.code.get_code(self.ip);
        self.ip += 1;

        byte
    }

    fn binary_op<F>(&mut self, op: F)
    where
        F: FnOnce(Value, Value) -> Value,
    {
        let b = self.stack.pop();
        let a = self.stack.pop();
        self.stack.push(op(a, b));
    }
}
