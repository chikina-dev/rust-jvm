use std::convert::TryFrom;

use crate::vm::{
    class::{RuntimeFieldRef, RuntimeMethodRef},
    error::{UnsupportedFeature, VmError},
    frame::Frame,
    ids::{CpIndex, ObjectRef, ThreadId},
    instruction::{DecodedInstruction, DecodedInstructionKind},
    memory::{Array, HeapEntry, VirtualMemory},
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
            DecodedInstructionKind::AConstNull => {
                self.push(memory, Value::null())?;
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
            DecodedInstructionKind::Ldc(index) => self.ldc(memory, instruction, *index),
            DecodedInstructionKind::ILoad(index) => {
                let value = self.local(memory, *index)?;
                self.push(memory, value)?;
                self.set_pc(memory, instruction.next_pc)?;
                Ok(StepResult::Continue)
            }
            DecodedInstructionKind::ALoad(index) => {
                let value = self.local(memory, *index)?;
                match value {
                    Value::Ref(_) => {
                        self.push(memory, value)?;
                        self.set_pc(memory, instruction.next_pc)?;
                        Ok(StepResult::Continue)
                    }
                    value => Err(VmError::VerificationError(format!(
                        "expected reference in local {index}, found {value:?}"
                    ))),
                }
            }
            DecodedInstructionKind::AALoad => self.ref_array_load(memory, instruction),
            DecodedInstructionKind::IALoad => self.int_array_load(memory, instruction),
            DecodedInstructionKind::IStore(index) => {
                let value = self.pop_int(memory)?;
                self.set_local(memory, *index, Value::Int(value))?;
                self.set_pc(memory, instruction.next_pc)?;
                Ok(StepResult::Continue)
            }
            DecodedInstructionKind::AStore(index) => {
                let value = self.pop_value(memory)?;
                match value {
                    Value::Ref(_) => {
                        self.set_local(memory, *index, value)?;
                        self.set_pc(memory, instruction.next_pc)?;
                        Ok(StepResult::Continue)
                    }
                    value => Err(VmError::VerificationError(format!(
                        "expected reference on operand stack, found {value:?}"
                    ))),
                }
            }
            DecodedInstructionKind::AAStore => self.ref_array_store(memory, instruction),
            DecodedInstructionKind::IAStore => self.int_array_store(memory, instruction),
            DecodedInstructionKind::Dup => {
                let value = self
                    .current_frame_mut(memory)?
                    .operand_stack
                    .last()
                    .cloned()
                    .ok_or_else(|| {
                        VmError::VerificationError("operand stack underflow".to_string())
                    })?;
                self.push(memory, value)?;
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
            DecodedInstructionKind::InvokeStatic(index) => {
                self.invoke_static_instruction(memory, instruction, *index)
            }
            DecodedInstructionKind::InvokeSpecial(index) => {
                self.invoke_special_instruction(memory, instruction, *index)
            }
            DecodedInstructionKind::InvokeVirtual(index) => {
                self.invoke_virtual_instruction(memory, instruction, *index)
            }
            DecodedInstructionKind::New(index) => self.new_object(memory, instruction, *index),
            DecodedInstructionKind::ANewArray(index) => {
                self.new_ref_array(memory, instruction, *index)
            }
            DecodedInstructionKind::NewArray(atype) => self.new_array(memory, instruction, *atype),
            DecodedInstructionKind::ArrayLength => self.array_length(memory, instruction),
            DecodedInstructionKind::GetStatic(index) => {
                self.get_static(memory, instruction, *index)
            }
            DecodedInstructionKind::GetField(index) => self.get_field(memory, instruction, *index),
            DecodedInstructionKind::PutField(index) => self.put_field(memory, instruction, *index),
            DecodedInstructionKind::IReturn => {
                let value = Value::Int(self.pop_int(memory)?);
                self.return_from_frame(memory, Some(value))
            }
            DecodedInstructionKind::AReturn => {
                let value = self.pop_ref(memory)?;
                self.return_from_frame(memory, Some(Value::Ref(value)))
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

    fn ldc(
        &self,
        memory: &mut VirtualMemory,
        instruction: &DecodedInstruction,
        index: CpIndex,
    ) -> Result<StepResult, VmError> {
        let value = self
            .current_class(memory)?
            .string_refs
            .get(&index)
            .cloned()
            .map(Value::String)
            .ok_or_else(|| VmError::ConstantResolution {
                index,
                reason: "unsupported ldc constant".to_string(),
            })?;
        self.push(memory, value)?;
        self.set_pc(memory, instruction.next_pc)?;
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

    fn int_array_load(
        &self,
        memory: &mut VirtualMemory,
        instruction: &DecodedInstruction,
    ) -> Result<StepResult, VmError> {
        let index = self.checked_array_index(self.pop_int(memory)?)?;
        let array_ref = self.require_object_ref(self.pop_ref(memory)?)?;
        let value = match memory.heap.get(array_ref) {
            Some(HeapEntry::Array(Array::Int(values))) => *values.get(index).ok_or_else(|| {
                VmError::RuntimeException("ArrayIndexOutOfBoundsException".to_string())
            })?,
            Some(HeapEntry::Array(_)) => {
                return Err(VmError::VerificationError(
                    "iaload expected int array".to_string(),
                ));
            }
            Some(_) => {
                return Err(VmError::VerificationError(
                    "reference does not point to an array".to_string(),
                ));
            }
            None => {
                return Err(VmError::RuntimeException(
                    "invalid array reference".to_string(),
                ));
            }
        };

        self.push(memory, Value::Int(value))?;
        self.set_pc(memory, instruction.next_pc)?;
        Ok(StepResult::Continue)
    }

    fn ref_array_load(
        &self,
        memory: &mut VirtualMemory,
        instruction: &DecodedInstruction,
    ) -> Result<StepResult, VmError> {
        let index = self.checked_array_index(self.pop_int(memory)?)?;
        let array_ref = self.require_object_ref(self.pop_ref(memory)?)?;
        let value = match memory.heap.get(array_ref) {
            Some(HeapEntry::Array(Array::Ref(values))) => *values.get(index).ok_or_else(|| {
                VmError::RuntimeException("ArrayIndexOutOfBoundsException".to_string())
            })?,
            Some(HeapEntry::Array(_)) => {
                return Err(VmError::VerificationError(
                    "aaload expected reference array".to_string(),
                ));
            }
            Some(_) => {
                return Err(VmError::VerificationError(
                    "reference does not point to an array".to_string(),
                ));
            }
            None => {
                return Err(VmError::RuntimeException(
                    "invalid array reference".to_string(),
                ));
            }
        };

        self.push(memory, Value::Ref(value))?;
        self.set_pc(memory, instruction.next_pc)?;
        Ok(StepResult::Continue)
    }

    fn int_array_store(
        &self,
        memory: &mut VirtualMemory,
        instruction: &DecodedInstruction,
    ) -> Result<StepResult, VmError> {
        let value = self.pop_int(memory)?;
        let index = self.checked_array_index(self.pop_int(memory)?)?;
        let array_ref = self.require_object_ref(self.pop_ref(memory)?)?;

        match memory.heap.get_mut(array_ref) {
            Some(HeapEntry::Array(Array::Int(values))) => {
                let slot = values.get_mut(index).ok_or_else(|| {
                    VmError::RuntimeException("ArrayIndexOutOfBoundsException".to_string())
                })?;
                *slot = value;
            }
            Some(HeapEntry::Array(_)) => {
                return Err(VmError::VerificationError(
                    "iastore expected int array".to_string(),
                ));
            }
            Some(_) => {
                return Err(VmError::VerificationError(
                    "reference does not point to an array".to_string(),
                ));
            }
            None => {
                return Err(VmError::RuntimeException(
                    "invalid array reference".to_string(),
                ));
            }
        }

        self.set_pc(memory, instruction.next_pc)?;
        Ok(StepResult::Continue)
    }

    fn ref_array_store(
        &self,
        memory: &mut VirtualMemory,
        instruction: &DecodedInstruction,
    ) -> Result<StepResult, VmError> {
        let value = self.pop_ref(memory)?;
        let index = self.checked_array_index(self.pop_int(memory)?)?;
        let array_ref = self.require_object_ref(self.pop_ref(memory)?)?;

        match memory.heap.get_mut(array_ref) {
            Some(HeapEntry::Array(Array::Ref(values))) => {
                let slot = values.get_mut(index).ok_or_else(|| {
                    VmError::RuntimeException("ArrayIndexOutOfBoundsException".to_string())
                })?;
                *slot = value;
            }
            Some(HeapEntry::Array(_)) => {
                return Err(VmError::VerificationError(
                    "aastore expected reference array".to_string(),
                ));
            }
            Some(_) => {
                return Err(VmError::VerificationError(
                    "reference does not point to an array".to_string(),
                ));
            }
            None => {
                return Err(VmError::RuntimeException(
                    "invalid array reference".to_string(),
                ));
            }
        }

        self.set_pc(memory, instruction.next_pc)?;
        Ok(StepResult::Continue)
    }

    fn new_object(
        &self,
        memory: &mut VirtualMemory,
        instruction: &DecodedInstruction,
        index: CpIndex,
    ) -> Result<StepResult, VmError> {
        let class_name = self.resolve_class_ref(memory, index)?;
        let class_id = memory
            .method_area
            .class_id(&class_name)
            .ok_or_else(|| VmError::ClassNotFound(class_name.clone()))?;
        let fields = memory
            .method_area
            .class(class_id)
            .ok_or_else(|| VmError::ClassNotFound(class_name.clone()))?
            .default_instance_fields();
        let reference = memory.heap.allocate_object(class_id, fields);
        self.push(memory, Value::Ref(Some(reference)))?;
        self.set_pc(memory, instruction.next_pc)?;
        Ok(StepResult::Continue)
    }

    fn new_array(
        &self,
        memory: &mut VirtualMemory,
        instruction: &DecodedInstruction,
        atype: u8,
    ) -> Result<StepResult, VmError> {
        let count = self.pop_int(memory)?;
        if count < 0 {
            return Err(VmError::RuntimeException(
                "NegativeArraySizeException".to_string(),
            ));
        }
        let count = usize::try_from(count)
            .map_err(|_| VmError::RuntimeException("NegativeArraySizeException".to_string()))?;

        let array = match atype {
            10 => Array::Int(vec![0; count]),
            _ => {
                return Err(VmError::UnsupportedFeature(UnsupportedFeature::Opcode {
                    opcode: instruction.opcode,
                    mnemonic: "newarray",
                }));
            }
        };
        let reference = memory.heap.allocate_array(array);
        self.push(memory, Value::Ref(Some(reference)))?;
        self.set_pc(memory, instruction.next_pc)?;
        Ok(StepResult::Continue)
    }

    fn new_ref_array(
        &self,
        memory: &mut VirtualMemory,
        instruction: &DecodedInstruction,
        index: CpIndex,
    ) -> Result<StepResult, VmError> {
        let _component_class_name = self.resolve_class_ref(memory, index)?;
        let count = self.pop_int(memory)?;
        if count < 0 {
            return Err(VmError::RuntimeException(
                "NegativeArraySizeException".to_string(),
            ));
        }
        let count = usize::try_from(count)
            .map_err(|_| VmError::RuntimeException("NegativeArraySizeException".to_string()))?;

        let reference = memory.heap.allocate_array(Array::Ref(vec![None; count]));
        self.push(memory, Value::Ref(Some(reference)))?;
        self.set_pc(memory, instruction.next_pc)?;
        Ok(StepResult::Continue)
    }

    fn array_length(
        &self,
        memory: &mut VirtualMemory,
        instruction: &DecodedInstruction,
    ) -> Result<StepResult, VmError> {
        let array_ref = self.require_object_ref(self.pop_ref(memory)?)?;
        let length = match memory.heap.get(array_ref) {
            Some(HeapEntry::Array(Array::Int(values))) => values.len(),
            Some(HeapEntry::Array(Array::Ref(values))) => values.len(),
            Some(HeapEntry::Array(Array::Values(values))) => values.len(),
            Some(_) => {
                return Err(VmError::VerificationError(
                    "reference does not point to an array".to_string(),
                ));
            }
            None => {
                return Err(VmError::RuntimeException(
                    "invalid array reference".to_string(),
                ));
            }
        };
        let length = i32::try_from(length).map_err(|_| {
            VmError::InternalError("array length does not fit into int".to_string())
        })?;
        self.push(memory, Value::Int(length))?;
        self.set_pc(memory, instruction.next_pc)?;
        Ok(StepResult::Continue)
    }

    fn invoke_static_instruction(
        &self,
        memory: &mut VirtualMemory,
        instruction: &DecodedInstruction,
        index: CpIndex,
    ) -> Result<StepResult, VmError> {
        let method_ref = self.resolve_method_ref(memory, index)?;
        let class_id = memory
            .method_area
            .class_id(&method_ref.class_name)
            .ok_or_else(|| VmError::ClassNotFound(method_ref.class_name.clone()))?;
        let (method_id, max_locals, code, parameter_count) = {
            let class = memory
                .method_area
                .class(class_id)
                .ok_or_else(|| VmError::ClassNotFound(method_ref.class_name.clone()))?;
            let method = class
                .find_method(&method_ref.name, &method_ref.descriptor)
                .ok_or_else(|| VmError::NoSuchMethod {
                    class: method_ref.class_name.clone(),
                    name: method_ref.name.clone(),
                    descriptor: method_ref.descriptor.clone(),
                })?;
            let code = method.code.clone().ok_or_else(|| {
                VmError::UnsupportedFeature(UnsupportedFeature::NativeMethod {
                    class: method_ref.class_name.clone(),
                    name: method_ref.name.clone(),
                    descriptor: method_ref.descriptor.clone(),
                })
            })?;

            (
                method.id,
                method.max_locals as usize,
                code,
                method.descriptor.parameters.len(),
            )
        };

        let mut args = Vec::with_capacity(parameter_count);
        for _ in 0..parameter_count {
            args.push(self.pop_value(memory)?);
        }
        args.reverse();

        self.set_pc(memory, instruction.next_pc)?;

        let mut frame = Frame::new(class_id, method_id, max_locals, code);
        for (index, value) in args.into_iter().enumerate() {
            let local = frame.locals.get_mut(index).ok_or_else(|| {
                VmError::VerificationError(format!("local {index} does not exist"))
            })?;
            *local = value;
        }

        memory
            .thread_mut(self.current_thread)
            .ok_or_else(|| {
                VmError::InternalError(format!("thread {:?} does not exist", self.current_thread))
            })?
            .push_frame(frame);

        Ok(StepResult::Continue)
    }

    fn invoke_special_instruction(
        &self,
        memory: &mut VirtualMemory,
        instruction: &DecodedInstruction,
        index: CpIndex,
    ) -> Result<StepResult, VmError> {
        let method_ref = self.resolve_method_ref(memory, index)?;
        let parameter_count =
            crate::vm::descriptor::parse_method_descriptor(&method_ref.descriptor)?
                .parameters
                .len();

        if method_ref.class_name == "java/lang/Object"
            && method_ref.name == "<init>"
            && method_ref.descriptor == "()V"
        {
            let object = self.pop_ref(memory)?;
            if object.is_none() {
                return Err(VmError::RuntimeException(
                    "NullPointerException".to_string(),
                ));
            }
            self.set_pc(memory, instruction.next_pc)?;
            return Ok(StepResult::Continue);
        }

        let class_id = memory
            .method_area
            .class_id(&method_ref.class_name)
            .ok_or_else(|| VmError::ClassNotFound(method_ref.class_name.clone()))?;
        let (method_id, max_locals, code) = {
            let class = memory
                .method_area
                .class(class_id)
                .ok_or_else(|| VmError::ClassNotFound(method_ref.class_name.clone()))?;
            let method = class
                .find_method(&method_ref.name, &method_ref.descriptor)
                .ok_or_else(|| VmError::NoSuchMethod {
                    class: method_ref.class_name.clone(),
                    name: method_ref.name.clone(),
                    descriptor: method_ref.descriptor.clone(),
                })?;
            let code = method.code.clone().ok_or_else(|| {
                VmError::UnsupportedFeature(UnsupportedFeature::NativeMethod {
                    class: method_ref.class_name.clone(),
                    name: method_ref.name.clone(),
                    descriptor: method_ref.descriptor.clone(),
                })
            })?;

            (method.id, method.max_locals as usize, code)
        };

        let mut args = Vec::with_capacity(parameter_count);
        for _ in 0..parameter_count {
            args.push(self.pop_value(memory)?);
        }
        args.reverse();
        let object = self.pop_ref(memory)?;
        if object.is_none() {
            return Err(VmError::RuntimeException(
                "NullPointerException".to_string(),
            ));
        }

        self.set_pc(memory, instruction.next_pc)?;

        let mut frame = Frame::new(class_id, method_id, max_locals, code);
        let receiver = frame
            .locals
            .get_mut(0)
            .ok_or_else(|| VmError::VerificationError("local 0 does not exist".to_string()))?;
        *receiver = Value::Ref(object);
        for (index, value) in args.into_iter().enumerate() {
            let local_index = index + 1;
            let local = frame.locals.get_mut(local_index).ok_or_else(|| {
                VmError::VerificationError(format!("local {local_index} does not exist"))
            })?;
            *local = value;
        }

        memory
            .thread_mut(self.current_thread)
            .ok_or_else(|| {
                VmError::InternalError(format!("thread {:?} does not exist", self.current_thread))
            })?
            .push_frame(frame);

        Ok(StepResult::Continue)
    }

    fn invoke_virtual_instruction(
        &self,
        memory: &mut VirtualMemory,
        instruction: &DecodedInstruction,
        index: CpIndex,
    ) -> Result<StepResult, VmError> {
        let method_ref = self.resolve_method_ref(memory, index)?;
        if method_ref.class_name == "java/io/PrintStream"
            && (method_ref.name == "println" || method_ref.name == "print")
        {
            let descriptor =
                crate::vm::descriptor::parse_method_descriptor(&method_ref.descriptor)?;
            let mut args = Vec::with_capacity(descriptor.parameters.len());
            for _ in 0..descriptor.parameters.len() {
                args.push(self.pop_value(memory)?);
            }
            args.reverse();

            let receiver = self.pop_value(memory)?;
            let is_stderr = match receiver {
                Value::NativeStdout => false,
                Value::NativeStderr => true,
                value => {
                    return Err(VmError::VerificationError(format!(
                        "println expected PrintStream receiver, found {value:?}"
                    )));
                }
            };
            let text = if let Some(value) = args.first() {
                printable_value(value)?
            } else {
                String::new()
            };
            let text = if method_ref.name == "println" {
                format!("{text}\n")
            } else {
                text
            };

            if is_stderr {
                memory.stderr.push(text);
            } else {
                memory.stdout.push(text);
            }
            self.set_pc(memory, instruction.next_pc)?;
            return Ok(StepResult::Continue);
        }

        Err(VmError::UnsupportedFeature(
            UnsupportedFeature::NativeMethod {
                class: method_ref.class_name,
                name: method_ref.name,
                descriptor: method_ref.descriptor,
            },
        ))
    }

    fn resolve_method_ref(
        &self,
        memory: &VirtualMemory,
        index: CpIndex,
    ) -> Result<RuntimeMethodRef, VmError> {
        let thread = memory.thread(self.current_thread).ok_or_else(|| {
            VmError::InternalError(format!("thread {:?} does not exist", self.current_thread))
        })?;
        let frame = thread.current_frame().ok_or_else(|| {
            VmError::InternalError(format!(
                "thread {:?} has no current frame",
                self.current_thread
            ))
        })?;
        let class = memory
            .method_area
            .class(frame.class_id)
            .ok_or_else(|| VmError::ClassNotFound(format!("{:?}", frame.class_id)))?;

        class
            .method_refs
            .get(&index)
            .cloned()
            .ok_or_else(|| VmError::ConstantResolution {
                index,
                reason: "expected Methodref constant".to_string(),
            })
    }

    fn get_field(
        &self,
        memory: &mut VirtualMemory,
        instruction: &DecodedInstruction,
        index: CpIndex,
    ) -> Result<StepResult, VmError> {
        let field_ref = self.resolve_field_ref(memory, index)?;
        let object = self.require_object_ref(self.pop_ref(memory)?)?;
        let value = self.object_field(memory, object, &field_ref)?.clone();
        self.push(memory, value)?;
        self.set_pc(memory, instruction.next_pc)?;
        Ok(StepResult::Continue)
    }

    fn put_field(
        &self,
        memory: &mut VirtualMemory,
        instruction: &DecodedInstruction,
        index: CpIndex,
    ) -> Result<StepResult, VmError> {
        let field_ref = self.resolve_field_ref(memory, index)?;
        let value = self.pop_value(memory)?;
        let object = self.require_object_ref(self.pop_ref(memory)?)?;
        let field = self.object_field_mut(memory, object, &field_ref)?;
        *field = value;
        self.set_pc(memory, instruction.next_pc)?;
        Ok(StepResult::Continue)
    }

    fn get_static(
        &self,
        memory: &mut VirtualMemory,
        instruction: &DecodedInstruction,
        index: CpIndex,
    ) -> Result<StepResult, VmError> {
        let field_ref = self.resolve_field_ref(memory, index)?;
        let value = match (
            field_ref.class_name.as_str(),
            field_ref.name.as_str(),
            field_ref.descriptor.as_str(),
        ) {
            ("java/lang/System", "out", "Ljava/io/PrintStream;") => Value::NativeStdout,
            ("java/lang/System", "err", "Ljava/io/PrintStream;") => Value::NativeStderr,
            _ => {
                return Err(VmError::UnsupportedFeature(
                    UnsupportedFeature::NativeMethod {
                        class: field_ref.class_name,
                        name: field_ref.name,
                        descriptor: field_ref.descriptor,
                    },
                ));
            }
        };
        self.push(memory, value)?;
        self.set_pc(memory, instruction.next_pc)?;
        Ok(StepResult::Continue)
    }

    fn resolve_class_ref(&self, memory: &VirtualMemory, index: CpIndex) -> Result<String, VmError> {
        let class = self.current_class(memory)?;
        class
            .class_refs
            .get(&index)
            .cloned()
            .ok_or_else(|| VmError::ConstantResolution {
                index,
                reason: "expected Class constant".to_string(),
            })
    }

    fn resolve_field_ref(
        &self,
        memory: &VirtualMemory,
        index: CpIndex,
    ) -> Result<RuntimeFieldRef, VmError> {
        let class = self.current_class(memory)?;
        class
            .field_refs
            .get(&index)
            .cloned()
            .ok_or_else(|| VmError::ConstantResolution {
                index,
                reason: "expected Fieldref constant".to_string(),
            })
    }

    fn current_class<'a>(
        &self,
        memory: &'a VirtualMemory,
    ) -> Result<&'a crate::vm::class::RuntimeClass, VmError> {
        let thread = memory.thread(self.current_thread).ok_or_else(|| {
            VmError::InternalError(format!("thread {:?} does not exist", self.current_thread))
        })?;
        let frame = thread.current_frame().ok_or_else(|| {
            VmError::InternalError(format!(
                "thread {:?} has no current frame",
                self.current_thread
            ))
        })?;
        memory
            .method_area
            .class(frame.class_id)
            .ok_or_else(|| VmError::ClassNotFound(format!("{:?}", frame.class_id)))
    }

    fn object_field<'a>(
        &self,
        memory: &'a VirtualMemory,
        object_ref: ObjectRef,
        field_ref: &RuntimeFieldRef,
    ) -> Result<&'a Value, VmError> {
        let object = match memory.heap.get(object_ref) {
            Some(HeapEntry::Object(object)) => object,
            Some(_) => {
                return Err(VmError::VerificationError(
                    "reference does not point to an object".to_string(),
                ));
            }
            None => {
                return Err(VmError::RuntimeException(
                    "invalid object reference".to_string(),
                ));
            }
        };
        let class = memory
            .method_area
            .class(object.class_id)
            .ok_or_else(|| VmError::ClassNotFound(format!("{:?}", object.class_id)))?;
        let field = class
            .find_field(&field_ref.name, &field_ref.descriptor)
            .ok_or_else(|| VmError::NoSuchField {
                class: field_ref.class_name.clone(),
                name: field_ref.name.clone(),
                descriptor: field_ref.descriptor.clone(),
            })?;
        object.fields.get(field.id.0).ok_or_else(|| {
            VmError::VerificationError(format!("field {:?} does not exist", field.id))
        })
    }

    fn object_field_mut<'a>(
        &self,
        memory: &'a mut VirtualMemory,
        object_ref: ObjectRef,
        field_ref: &RuntimeFieldRef,
    ) -> Result<&'a mut Value, VmError> {
        let field_id = {
            let object = match memory.heap.get(object_ref) {
                Some(HeapEntry::Object(object)) => object,
                Some(_) => {
                    return Err(VmError::VerificationError(
                        "reference does not point to an object".to_string(),
                    ));
                }
                None => {
                    return Err(VmError::RuntimeException(
                        "invalid object reference".to_string(),
                    ));
                }
            };
            let class = memory
                .method_area
                .class(object.class_id)
                .ok_or_else(|| VmError::ClassNotFound(format!("{:?}", object.class_id)))?;
            class
                .find_field(&field_ref.name, &field_ref.descriptor)
                .ok_or_else(|| VmError::NoSuchField {
                    class: field_ref.class_name.clone(),
                    name: field_ref.name.clone(),
                    descriptor: field_ref.descriptor.clone(),
                })?
                .id
        };

        match memory.heap.get_mut(object_ref) {
            Some(HeapEntry::Object(object)) => object.fields.get_mut(field_id.0).ok_or_else(|| {
                VmError::VerificationError(format!("field {:?} does not exist", field_id))
            }),
            Some(_) => Err(VmError::VerificationError(
                "reference does not point to an object".to_string(),
            )),
            None => Err(VmError::RuntimeException(
                "invalid object reference".to_string(),
            )),
        }
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

    fn pop_value(&self, memory: &mut VirtualMemory) -> Result<Value, VmError> {
        self.current_frame_mut(memory)?
            .operand_stack
            .pop()
            .ok_or_else(|| VmError::VerificationError("operand stack underflow".to_string()))
    }

    fn pop_ref(&self, memory: &mut VirtualMemory) -> Result<Option<ObjectRef>, VmError> {
        match self.pop_value(memory)? {
            Value::Ref(value) => Ok(value),
            value => Err(VmError::VerificationError(format!(
                "expected reference on operand stack, found {value:?}"
            ))),
        }
    }

    fn require_object_ref(&self, value: Option<ObjectRef>) -> Result<ObjectRef, VmError> {
        value.ok_or_else(|| VmError::RuntimeException("NullPointerException".to_string()))
    }

    fn checked_array_index(&self, index: i32) -> Result<usize, VmError> {
        if index < 0 {
            return Err(VmError::RuntimeException(
                "ArrayIndexOutOfBoundsException".to_string(),
            ));
        }
        usize::try_from(index)
            .map_err(|_| VmError::RuntimeException("ArrayIndexOutOfBoundsException".to_string()))
    }

    fn pop_int(&self, memory: &mut VirtualMemory) -> Result<i32, VmError> {
        match self.pop_value(memory)? {
            Value::Int(value) => Ok(value),
            value => Err(VmError::VerificationError(format!(
                "expected int on operand stack, found {value:?}"
            ))),
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

fn printable_value(value: &Value) -> Result<String, VmError> {
    match value {
        Value::Int(value) => Ok(value.to_string()),
        Value::Long(value) => Ok(value.to_string()),
        Value::Float(value) => Ok(value.to_string()),
        Value::Double(value) => Ok(value.to_string()),
        Value::String(value) => Ok(value.clone()),
        Value::Ref(None) => Ok("null".to_string()),
        value => Err(VmError::UnsupportedFeature(
            UnsupportedFeature::NativeMethod {
                class: "java/io/PrintStream".to_string(),
                name: "println".to_string(),
                descriptor: format!("{value:?}"),
            },
        )),
    }
}
