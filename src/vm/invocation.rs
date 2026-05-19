use crate::vm::{
    error::{UnsupportedFeature, VmError},
    frame::Frame,
    ids::{ClassId, MethodId, ObjectRef},
    instruction::DecodedInstruction,
    memory::VirtualMemory,
    value::Value,
};

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedMethod {
    pub class_id: ClassId,
    pub method_id: MethodId,
    pub class_name: String,
    pub name: String,
    pub descriptor: String,
    pub max_locals: usize,
    pub parameter_count: usize,
    pub code: Vec<DecodedInstruction>,
}

impl ResolvedMethod {
    pub fn resolve(
        memory: &VirtualMemory,
        class_name: &str,
        name: &str,
        descriptor: &str,
    ) -> Result<Self, VmError> {
        let class_id = memory
            .method_area
            .class_id(class_name)
            .ok_or_else(|| VmError::ClassNotFound(class_name.to_string()))?;
        let class = memory
            .method_area
            .class(class_id)
            .ok_or_else(|| VmError::ClassNotFound(class_name.to_string()))?;
        let method = class
            .find_method(name, descriptor)
            .ok_or_else(|| VmError::NoSuchMethod {
                class: class_name.to_string(),
                name: name.to_string(),
                descriptor: descriptor.to_string(),
            })?;
        let code = method.code.clone().ok_or_else(|| {
            VmError::UnsupportedFeature(UnsupportedFeature::NativeMethod {
                class: class_name.to_string(),
                name: name.to_string(),
                descriptor: descriptor.to_string(),
            })
        })?;

        Ok(Self {
            class_id,
            method_id: method.id,
            class_name: class_name.to_string(),
            name: name.to_string(),
            descriptor: descriptor.to_string(),
            max_locals: method.max_locals as usize,
            parameter_count: method.descriptor.parameters.len(),
            code,
        })
    }

    pub fn frame_with_args(&self, args: Vec<Value>) -> Result<Frame, VmError> {
        Frame::new_with_args(
            self.class_id,
            self.method_id,
            self.max_locals,
            self.code.clone(),
            args,
        )
    }

    pub fn frame_with_receiver_and_args(
        &self,
        receiver: Option<ObjectRef>,
        args: Vec<Value>,
    ) -> Result<Frame, VmError> {
        let locals = std::iter::once((0, Value::Ref(receiver))).chain(
            args.into_iter()
                .enumerate()
                .map(|(index, value)| (index + 1, value)),
        );
        Frame::new_with_locals(
            self.class_id,
            self.method_id,
            self.max_locals,
            self.code.clone(),
            locals,
        )
    }
}
