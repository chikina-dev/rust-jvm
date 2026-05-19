use crate::vm::{
    error::{UnsupportedFeature, VmError},
    ids::ThreadId,
    instruction::{DecodedInstruction, DecodedInstructionKind},
    memory::VirtualMemory,
    value::Value,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CpuState {
    Ready,
    Running,
    Halted,
    Trapped(VmError),
}

#[derive(Debug, Clone, PartialEq)]
pub enum StepResult {
    Continue,
    Return(Option<Value>),
    Halted,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VirtualCpu {
    pub current_thread: ThreadId,
    pub state: CpuState,
}

impl VirtualCpu {
    pub fn new(current_thread: ThreadId) -> Self {
        Self {
            current_thread,
            state: CpuState::Ready,
        }
    }

    pub fn step(&mut self, memory: &mut VirtualMemory) -> Result<StepResult, VmError> {
        self.state = CpuState::Running;

        let instruction = self.current_instruction(memory)?.clone();
        let result = self.execute(memory, &instruction);

        match &result {
            Ok(StepResult::Return(_)) | Ok(StepResult::Halted) => {
                self.state = CpuState::Halted;
            }
            Ok(StepResult::Continue) => {
                self.state = CpuState::Ready;
            }
            Err(error) => {
                self.state = CpuState::Trapped(error.clone());
            }
        }

        result
    }

    pub fn run_until_return(
        &mut self,
        memory: &mut VirtualMemory,
    ) -> Result<Option<Value>, VmError> {
        loop {
            match self.step(memory)? {
                StepResult::Continue => {}
                StepResult::Return(value) => return Ok(value),
                StepResult::Halted => return Ok(None),
            }
        }
    }

    fn current_instruction<'a>(
        &self,
        memory: &'a VirtualMemory,
    ) -> Result<&'a DecodedInstruction, VmError> {
        let thread = memory.thread(self.current_thread).ok_or_else(|| {
            VmError::InternalError(format!("thread {:?} does not exist", self.current_thread))
        })?;
        let frame = thread.current_frame().ok_or_else(|| {
            VmError::InternalError(format!(
                "thread {:?} has no current frame",
                self.current_thread
            ))
        })?;
        frame
            .instruction_at_pc()
            .ok_or_else(|| VmError::VerificationError(format!("no instruction at pc {}", frame.pc)))
    }

    fn execute(
        &mut self,
        memory: &mut VirtualMemory,
        instruction: &DecodedInstruction,
    ) -> Result<StepResult, VmError> {
        match &instruction.kind {
            DecodedInstructionKind::Nop => {
                self.set_pc(memory, instruction.next_pc)?;
                Ok(StepResult::Continue)
            }
            DecodedInstructionKind::IConst(value) => {
                self.push(memory, Value::Int(*value))?;
                self.set_pc(memory, instruction.next_pc)?;
                Ok(StepResult::Continue)
            }
            DecodedInstructionKind::BiPush(value) => {
                self.push(memory, Value::Int(*value as i32))?;
                self.set_pc(memory, instruction.next_pc)?;
                Ok(StepResult::Continue)
            }
            DecodedInstructionKind::SiPush(value) => {
                self.push(memory, Value::Int(*value as i32))?;
                self.set_pc(memory, instruction.next_pc)?;
                Ok(StepResult::Continue)
            }
            DecodedInstructionKind::ILoad(index) => {
                let value = self.local(memory, *index)?;
                self.push(memory, value)?;
                self.set_pc(memory, instruction.next_pc)?;
                Ok(StepResult::Continue)
            }
            DecodedInstructionKind::IStore(index) => {
                let value = self.pop_int(memory)?;
                self.set_local(memory, *index, Value::Int(value))?;
                self.set_pc(memory, instruction.next_pc)?;
                Ok(StepResult::Continue)
            }
            DecodedInstructionKind::IAdd => {
                self.binary_int(memory, instruction.next_pc, |a, b| a + b)
            }
            DecodedInstructionKind::ISub => {
                self.binary_int(memory, instruction.next_pc, |a, b| a - b)
            }
            DecodedInstructionKind::IMul => {
                self.binary_int(memory, instruction.next_pc, |a, b| a * b)
            }
            DecodedInstructionKind::IDiv => {
                let divisor = self.pop_int(memory)?;
                if divisor == 0 {
                    return Err(VmError::RuntimeException("/ by zero".to_string()));
                }
                let dividend = self.pop_int(memory)?;
                self.push(memory, Value::Int(dividend / divisor))?;
                self.set_pc(memory, instruction.next_pc)?;
                Ok(StepResult::Continue)
            }
            DecodedInstructionKind::IRem => {
                let divisor = self.pop_int(memory)?;
                if divisor == 0 {
                    return Err(VmError::RuntimeException("/ by zero".to_string()));
                }
                let dividend = self.pop_int(memory)?;
                self.push(memory, Value::Int(dividend % divisor))?;
                self.set_pc(memory, instruction.next_pc)?;
                Ok(StepResult::Continue)
            }
            DecodedInstructionKind::Goto(offset) => {
                self.set_branch_pc(memory, instruction.pc, *offset)?;
                Ok(StepResult::Continue)
            }
            DecodedInstructionKind::IfEq(offset) => {
                self.branch_if(memory, instruction, *offset, |value| value == 0)
            }
            DecodedInstructionKind::IfNe(offset) => {
                self.branch_if(memory, instruction, *offset, |value| value != 0)
            }
            DecodedInstructionKind::IfLt(offset) => {
                self.branch_if(memory, instruction, *offset, |value| value < 0)
            }
            DecodedInstructionKind::IfGe(offset) => {
                self.branch_if(memory, instruction, *offset, |value| value >= 0)
            }
            DecodedInstructionKind::IfGt(offset) => {
                self.branch_if(memory, instruction, *offset, |value| value > 0)
            }
            DecodedInstructionKind::IfLe(offset) => {
                self.branch_if(memory, instruction, *offset, |value| value <= 0)
            }
            DecodedInstructionKind::IfICmpEq(offset) => {
                self.branch_if_icmp(memory, instruction, *offset, |lhs, rhs| lhs == rhs)
            }
            DecodedInstructionKind::IfICmpNe(offset) => {
                self.branch_if_icmp(memory, instruction, *offset, |lhs, rhs| lhs != rhs)
            }
            DecodedInstructionKind::IfICmpLt(offset) => {
                self.branch_if_icmp(memory, instruction, *offset, |lhs, rhs| lhs < rhs)
            }
            DecodedInstructionKind::IfICmpGe(offset) => {
                self.branch_if_icmp(memory, instruction, *offset, |lhs, rhs| lhs >= rhs)
            }
            DecodedInstructionKind::IfICmpGt(offset) => {
                self.branch_if_icmp(memory, instruction, *offset, |lhs, rhs| lhs > rhs)
            }
            DecodedInstructionKind::IfICmpLe(offset) => {
                self.branch_if_icmp(memory, instruction, *offset, |lhs, rhs| lhs <= rhs)
            }
            DecodedInstructionKind::IInc { index, value } => {
                let current = match self.local(memory, *index)? {
                    Value::Int(value) => value,
                    value => {
                        return Err(VmError::VerificationError(format!(
                            "expected int in local {index}, found {value:?}"
                        )));
                    }
                };
                self.set_local(memory, *index, Value::Int(current + *value as i32))?;
                self.set_pc(memory, instruction.next_pc)?;
                Ok(StepResult::Continue)
            }
            DecodedInstructionKind::IReturn => {
                let value = Value::Int(self.pop_int(memory)?);
                self.return_from_frame(memory, Some(value))
            }
            DecodedInstructionKind::Return => self.return_from_frame(memory, None),
            DecodedInstructionKind::Unsupported { mnemonic } => {
                Err(VmError::UnsupportedFeature(UnsupportedFeature::Opcode {
                    opcode: instruction.opcode,
                    mnemonic,
                }))
            }
            DecodedInstructionKind::Unknown { opcode } => Err(VmError::VerificationError(format!(
                "unknown opcode 0x{opcode:02x}"
            ))),
            other => Err(VmError::UnsupportedFeature(UnsupportedFeature::Opcode {
                opcode: instruction.opcode,
                mnemonic: other.mnemonic_fallback(),
            })),
        }
    }

    fn binary_int(
        &self,
        memory: &mut VirtualMemory,
        next_pc: u32,
        operation: impl FnOnce(i32, i32) -> i32,
    ) -> Result<StepResult, VmError> {
        let rhs = self.pop_int(memory)?;
        let lhs = self.pop_int(memory)?;
        self.push(memory, Value::Int(operation(lhs, rhs)))?;
        self.set_pc(memory, next_pc)?;
        Ok(StepResult::Continue)
    }

    fn branch_if(
        &self,
        memory: &mut VirtualMemory,
        instruction: &DecodedInstruction,
        offset: i16,
        predicate: impl FnOnce(i32) -> bool,
    ) -> Result<StepResult, VmError> {
        let value = self.pop_int(memory)?;
        if predicate(value) {
            self.set_branch_pc(memory, instruction.pc, offset)?;
        } else {
            self.set_pc(memory, instruction.next_pc)?;
        }
        Ok(StepResult::Continue)
    }

    fn branch_if_icmp(
        &self,
        memory: &mut VirtualMemory,
        instruction: &DecodedInstruction,
        offset: i16,
        predicate: impl FnOnce(i32, i32) -> bool,
    ) -> Result<StepResult, VmError> {
        let rhs = self.pop_int(memory)?;
        let lhs = self.pop_int(memory)?;
        if predicate(lhs, rhs) {
            self.set_branch_pc(memory, instruction.pc, offset)?;
        } else {
            self.set_pc(memory, instruction.next_pc)?;
        }
        Ok(StepResult::Continue)
    }

    fn return_from_frame(
        &self,
        memory: &mut VirtualMemory,
        value: Option<Value>,
    ) -> Result<StepResult, VmError> {
        let thread = memory.thread_mut(self.current_thread).ok_or_else(|| {
            VmError::InternalError(format!("thread {:?} does not exist", self.current_thread))
        })?;
        thread.pop_frame().ok_or_else(|| {
            VmError::InternalError(format!(
                "thread {:?} has no frame to return from",
                self.current_thread
            ))
        })?;

        if let Some(caller) = thread.current_frame_mut() {
            if let Some(value) = value.clone() {
                caller.operand_stack.push(value);
            }
            Ok(StepResult::Continue)
        } else {
            Ok(StepResult::Return(value))
        }
    }

    fn push(&self, memory: &mut VirtualMemory, value: Value) -> Result<(), VmError> {
        self.current_frame_mut(memory)?.operand_stack.push(value);
        Ok(())
    }

    fn pop_int(&self, memory: &mut VirtualMemory) -> Result<i32, VmError> {
        match self.current_frame_mut(memory)?.operand_stack.pop() {
            Some(Value::Int(value)) => Ok(value),
            Some(value) => Err(VmError::VerificationError(format!(
                "expected int on operand stack, found {value:?}"
            ))),
            None => Err(VmError::VerificationError(
                "operand stack underflow".to_string(),
            )),
        }
    }

    fn local(&self, memory: &mut VirtualMemory, index: u16) -> Result<Value, VmError> {
        self.current_frame_mut(memory)?
            .locals
            .get(index as usize)
            .cloned()
            .ok_or_else(|| VmError::VerificationError(format!("local {index} does not exist")))
    }

    fn set_local(
        &self,
        memory: &mut VirtualMemory,
        index: u16,
        value: Value,
    ) -> Result<(), VmError> {
        let local = self
            .current_frame_mut(memory)?
            .locals
            .get_mut(index as usize)
            .ok_or_else(|| VmError::VerificationError(format!("local {index} does not exist")))?;
        *local = value;
        Ok(())
    }

    fn set_pc(&self, memory: &mut VirtualMemory, pc: u32) -> Result<(), VmError> {
        self.current_frame_mut(memory)?.pc = pc;
        Ok(())
    }

    fn set_branch_pc(
        &self,
        memory: &mut VirtualMemory,
        current_pc: u32,
        offset: i16,
    ) -> Result<(), VmError> {
        let target = current_pc as i64 + offset as i64;
        if target < 0 || target > u32::MAX as i64 {
            return Err(VmError::VerificationError(format!(
                "branch target out of range: {target}"
            )));
        }
        self.set_pc(memory, target as u32)
    }

    fn current_frame_mut<'a>(
        &self,
        memory: &'a mut VirtualMemory,
    ) -> Result<&'a mut crate::vm::frame::Frame, VmError> {
        memory
            .thread_mut(self.current_thread)
            .ok_or_else(|| {
                VmError::InternalError(format!("thread {:?} does not exist", self.current_thread))
            })?
            .current_frame_mut()
            .ok_or_else(|| {
                VmError::InternalError(format!(
                    "thread {:?} has no current frame",
                    self.current_thread
                ))
            })
    }
}

trait MnemonicFallback {
    fn mnemonic_fallback(&self) -> &'static str;
}

impl MnemonicFallback for DecodedInstructionKind {
    fn mnemonic_fallback(&self) -> &'static str {
        match self {
            DecodedInstructionKind::Ldc(_) => "ldc",
            DecodedInstructionKind::IfICmpEq(_) => "if_icmpeq",
            DecodedInstructionKind::IfICmpNe(_) => "if_icmpne",
            DecodedInstructionKind::IfICmpLt(_) => "if_icmplt",
            DecodedInstructionKind::IfICmpGe(_) => "if_icmpge",
            DecodedInstructionKind::IfICmpGt(_) => "if_icmpgt",
            DecodedInstructionKind::IfICmpLe(_) => "if_icmple",
            DecodedInstructionKind::IInc { .. } => "iinc",
            _ => "unsupported",
        }
    }
}
