use crate::vm::{
    error::VmError,
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

    pub fn new_with_args(
        class_id: ClassId,
        method_id: MethodId,
        max_locals: usize,
        code: Vec<DecodedInstruction>,
        args: impl IntoIterator<Item = Value>,
    ) -> Result<Self, VmError> {
        Self::new_with_locals(
            class_id,
            method_id,
            max_locals,
            code,
            args.into_iter().enumerate(),
        )
    }

    pub fn new_with_locals(
        class_id: ClassId,
        method_id: MethodId,
        max_locals: usize,
        code: Vec<DecodedInstruction>,
        locals: impl IntoIterator<Item = (usize, Value)>,
    ) -> Result<Self, VmError> {
        let mut frame = Self::new(class_id, method_id, max_locals, code);
        for (index, value) in locals {
            let local = frame.locals.get_mut(index).ok_or_else(|| {
                VmError::VerificationError(format!("local {index} does not exist"))
            })?;
            *local = value;
        }
        Ok(frame)
    }

    pub fn instruction_at_pc(&self) -> Option<&DecodedInstruction> {
        self.code
            .iter()
            .find(|instruction| instruction.pc == self.pc)
    }
}
