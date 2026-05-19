use std::collections::HashMap;

use crate::{
    structure::class::Constant,
    structure::class::{ClassFile, Method, MethodInfoAttribute},
    vm::{
        constant::ConstantPoolExt,
        descriptor::{MethodDescriptor, parse_method_descriptor},
        error::VmError,
        ids::{ClassId, ClassIndex, CpIndex, MethodId},
        instruction::{DecodedInstruction, decode_code},
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClassState {
    Loaded,
    Linked,
    Initializing,
    Initialized,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeClass {
    pub id: ClassId,
    pub name: String,
    pub methods: Vec<RuntimeMethod>,
    pub method_refs: HashMap<CpIndex, RuntimeMethodRef>,
    pub state: ClassState,
}

impl RuntimeClass {
    pub fn from_class_file(id: ClassId, class_file: &ClassFile) -> Result<Self, VmError> {
        let name = class_file
            .constant_pool
            .class_name(ClassIndex(class_file.this_class))?;
        let methods = class_file
            .methods
            .methods
            .iter()
            .enumerate()
            .map(|(index, method)| RuntimeMethod::from_method(MethodId(index), class_file, method))
            .collect::<Result<Vec<_>, _>>()?;
        let method_refs = collect_method_refs(class_file)?;

        Ok(Self {
            id,
            name,
            methods,
            method_refs,
            state: ClassState::Loaded,
        })
    }

    pub fn find_method(&self, name: &str, descriptor: &str) -> Option<&RuntimeMethod> {
        self.methods
            .iter()
            .find(|method| method.name == name && method.descriptor_source == descriptor)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeMethodRef {
    pub class_name: String,
    pub name: String,
    pub descriptor: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeMethod {
    pub id: MethodId,
    pub name: String,
    pub descriptor_source: String,
    pub descriptor: MethodDescriptor,
    pub access_flags: u16,
    pub max_stack: u16,
    pub max_locals: u16,
    pub code: Option<Vec<DecodedInstruction>>,
}

impl RuntimeMethod {
    fn from_method(id: MethodId, class_file: &ClassFile, method: &Method) -> Result<Self, VmError> {
        let name = class_file.constant_pool.utf8(CpIndex(method.name_index))?;
        let descriptor_source = class_file
            .constant_pool
            .utf8(CpIndex(method.descriptor_index))?;
        let descriptor = parse_method_descriptor(&descriptor_source)?;

        let mut max_stack = 0;
        let mut max_locals = 0;
        let mut decoded_code = None;

        for attribute in &method.attributes.attributes {
            if let MethodInfoAttribute::Code(code) = attribute {
                max_stack = code.max_stack;
                max_locals = code.max_locals;
                decoded_code = Some(decode_code(&code.code)?);
            }
        }

        Ok(Self {
            id,
            name,
            descriptor_source,
            descriptor,
            access_flags: method.access_flags,
            max_stack,
            max_locals,
            code: decoded_code,
        })
    }
}

fn collect_method_refs(
    class_file: &ClassFile,
) -> Result<HashMap<CpIndex, RuntimeMethodRef>, VmError> {
    let mut refs = HashMap::new();

    for (zero_based_index, constant) in class_file.constant_pool.constants.iter().enumerate() {
        if let Constant::Methodref {
            class_index,
            name_and_type_index,
        } = constant
        {
            let cp_index = CpIndex((zero_based_index + 1) as u16);
            let class_name = class_file
                .constant_pool
                .class_name(ClassIndex(*class_index))?;
            let (name, descriptor) = class_file
                .constant_pool
                .name_and_type(CpIndex(*name_and_type_index))?;

            refs.insert(
                cp_index,
                RuntimeMethodRef {
                    class_name,
                    name,
                    descriptor,
                },
            );
        }
    }

    Ok(refs)
}
