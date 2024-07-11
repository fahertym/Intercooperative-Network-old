// src/vm/coop_vm.rs

use super::opcode::Opcode;

pub struct CoopVM {
    stack: Vec<Value>,
    memory: std::collections::HashMap<String, Value>,
    program: Vec<Opcode>,
    pc: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
}

impl CoopVM {
    pub fn new(program: Vec<Opcode>) -> Self {
        CoopVM {
            stack: Vec::new(),
            memory: std::collections::HashMap::new(),
            program,
            pc: 0,
        }
    }

    pub fn run(&mut self) -> Result<(), String> {
        while self.pc < self.program.len() {
            self.execute_instruction()?;
            self.pc += 1;
        }
        Ok(())
    }

    fn execute_instruction(&mut self) -> Result<(), String> {
        // Implement instruction execution logic
        Ok(())
    }

    pub fn load_program(&mut self, program: Vec<Opcode>) {
        self.program = program;
        self.pc = 0;
    }

    pub fn get_stack(&self) -> &Vec<Value> {
        &self.stack
    }

    pub fn get_memory(&self) -> &std::collections::HashMap<String, Value> {
        &self.memory
    }
}