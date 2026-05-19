use crate::vm::{
    ids::{ClassId, MethodId},
    instruction::DecodedInstruction,
    value::Value,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Frame {
    pub class_id: ClassId,
    pub method_id: MethodId,
    pub pc: u32,
    pub locals: Vec<Value>,
    pub operand_stack: Vec<Value>,
    pub code: Vec<DecodedInstruction>,
}

impl Frame {
    pub fn new(
        class_id: ClassId,
        method_id: MethodId,
        max_locals: usize,
        code: Vec<DecodedInstruction>,
    ) -> Self {
        Self {
            class_id,
            method_id,
            pc: 0,
            locals: vec![Value::Int(0); max_locals],
            operand_stack: Vec::new(),
            code,
        }
    }

    pub fn instruction_at_pc(&self) -> Option<&DecodedInstruction> {
        self.code
            .iter()
            .find(|instruction| instruction.pc == self.pc)
    }
}
