use std::collections::HashMap;

use crate::{
    structure::class::{
        ClassFile, Constant, Field, FieldInfoAttribute, Method, MethodInfoAttribute,
    },
    vm::{
        constant::ConstantPoolExt,
        descriptor::{JavaType, MethodDescriptor, parse_field_descriptor, parse_method_descriptor},
        error::VmError,
        ids::{ClassId, ClassIndex, CpIndex, FieldId, MethodId},
        instruction::{DecodedInstruction, decode_code},
        value::Value,
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

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeClass {
    pub id: ClassId,
    pub name: String,
    pub fields: Vec<RuntimeField>,
    pub static_fields: Vec<Value>,
    pub class_refs: HashMap<CpIndex, String>,
    pub field_refs: HashMap<CpIndex, RuntimeFieldRef>,
    pub methods: Vec<RuntimeMethod>,
    pub method_refs: HashMap<CpIndex, RuntimeMethodRef>,
    pub string_refs: HashMap<CpIndex, String>,
    pub state: ClassState,
}

impl RuntimeClass {
    pub fn from_class_file(id: ClassId, class_file: &ClassFile) -> Result<Self, VmError> {
        let name = class_file
            .constant_pool
            .class_name(ClassIndex(class_file.this_class))?;
        let fields = class_file
            .fields
            .fields
            .iter()
            .enumerate()
            .map(|(index, field)| RuntimeField::from_field(FieldId(index), class_file, field))
            .collect::<Result<Vec<_>, _>>()?;
        let static_fields = fields
            .iter()
            .map(RuntimeField::static_initial_value)
            .collect();
        let methods = class_file
            .methods
            .methods
            .iter()
            .enumerate()
            .map(|(index, method)| RuntimeMethod::from_method(MethodId(index), class_file, method))
            .collect::<Result<Vec<_>, _>>()?;
        let class_refs = collect_class_refs(class_file)?;
        let field_refs = collect_field_refs(class_file)?;
        let method_refs = collect_method_refs(class_file)?;
        let string_refs = collect_string_refs(class_file)?;

        Ok(Self {
            id,
            name,
            fields,
            static_fields,
            class_refs,
            field_refs,
            methods,
            method_refs,
            string_refs,
            state: ClassState::Loaded,
        })
    }

    pub fn find_method(&self, name: &str, descriptor: &str) -> Option<&RuntimeMethod> {
        self.methods
            .iter()
            .find(|method| method.name == name && method.descriptor_source == descriptor)
    }

    pub fn find_field(&self, name: &str, descriptor: &str) -> Option<&RuntimeField> {
        self.fields
            .iter()
            .find(|field| field.name == name && field.descriptor_source == descriptor)
    }

    pub fn default_instance_fields(&self) -> Vec<Value> {
        self.fields
            .iter()
            .filter(|field| !field.is_static())
            .map(|field| default_value(&field.descriptor))
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeFieldRef {
    pub class_name: String,
    pub name: String,
    pub descriptor: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeMethodRef {
    pub class_name: String,
    pub name: String,
    pub descriptor: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeField {
    pub id: FieldId,
    pub name: String,
    pub descriptor_source: String,
    pub descriptor: JavaType,
    pub access_flags: u16,
    pub constant_value: Option<Value>,
}

impl RuntimeField {
    fn from_field(id: FieldId, class_file: &ClassFile, field: &Field) -> Result<Self, VmError> {
        let name = class_file.constant_pool.utf8(CpIndex(field.name_index))?;
        let descriptor_source = class_file
            .constant_pool
            .utf8(CpIndex(field.descriptor_index))?;
        let descriptor = parse_field_descriptor(&descriptor_source)?;
        let constant_value = field_constant_value(class_file, field, &descriptor)?;

        Ok(Self {
            id,
            name,
            descriptor_source,
            descriptor,
            access_flags: field.access_flags,
            constant_value,
        })
    }

    pub fn is_static(&self) -> bool {
        self.access_flags & 0x0008 != 0
    }

    fn static_initial_value(&self) -> Value {
        if self.is_static() {
            self.constant_value
                .clone()
                .unwrap_or_else(|| default_value(&self.descriptor))
        } else {
            Value::null()
        }
    }
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

fn field_constant_value(
    class_file: &ClassFile,
    field: &Field,
    descriptor: &JavaType,
) -> Result<Option<Value>, VmError> {
    for attribute in &field.attributes.attributes {
        if let FieldInfoAttribute::ConstantValue(constant_value) = attribute {
            let index = CpIndex(constant_value.constant_value_index);
            return constant_to_value(class_file, index, descriptor).map(Some);
        }
    }

    Ok(None)
}

fn constant_to_value(
    class_file: &ClassFile,
    index: CpIndex,
    descriptor: &JavaType,
) -> Result<Value, VmError> {
    let constant = class_file
        .constant_pool
        .constants
        .get(index.0.checked_sub(1).ok_or(
            crate::vm::error::ParseError::InvalidConstantPoolIndex(index.0),
        )? as usize)
        .ok_or(crate::vm::error::ParseError::InvalidConstantPoolIndex(
            index.0,
        ))?;

    match (descriptor, constant) {
        (
            JavaType::Byte | JavaType::Char | JavaType::Int | JavaType::Short | JavaType::Boolean,
            Constant::Integer { bytes },
        ) => Ok(Value::Int(*bytes as i32)),
        (
            JavaType::Long,
            Constant::Long {
                high_bytes,
                low_bytes,
            },
        ) => {
            let bits = ((*high_bytes as u64) << 32) | *low_bytes as u64;
            Ok(Value::Long(bits as i64))
        }
        (JavaType::Float, Constant::Float { bytes }) => Ok(Value::Float(f32::from_bits(*bytes))),
        (
            JavaType::Double,
            Constant::Double {
                high_bytes,
                low_bytes,
            },
        ) => {
            let bits = ((*high_bytes as u64) << 32) | *low_bytes as u64;
            Ok(Value::Double(f64::from_bits(bits)))
        }
        (JavaType::Object(class_name), Constant::String { string_index })
            if class_name == "java/lang/String" =>
        {
            class_file
                .constant_pool
                .utf8(CpIndex(*string_index))
                .map(Value::String)
                .map_err(Into::into)
        }
        _ => Err(VmError::ConstantResolution {
            index,
            reason: "ConstantValue type does not match field descriptor".to_string(),
        }),
    }
}

fn collect_class_refs(class_file: &ClassFile) -> Result<HashMap<CpIndex, String>, VmError> {
    let mut refs = HashMap::new();

    for (zero_based_index, constant) in class_file.constant_pool.constants.iter().enumerate() {
        if let Constant::Class { name_index } = constant {
            refs.insert(
                CpIndex((zero_based_index + 1) as u16),
                class_file.constant_pool.utf8(CpIndex(*name_index))?,
            );
        }
    }

    Ok(refs)
}

fn collect_field_refs(
    class_file: &ClassFile,
) -> Result<HashMap<CpIndex, RuntimeFieldRef>, VmError> {
    let mut refs = HashMap::new();

    for (zero_based_index, constant) in class_file.constant_pool.constants.iter().enumerate() {
        if let Constant::Fieldref {
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
                RuntimeFieldRef {
                    class_name,
                    name,
                    descriptor,
                },
            );
        }
    }

    Ok(refs)
}

fn collect_method_refs(
    class_file: &ClassFile,
) -> Result<HashMap<CpIndex, RuntimeMethodRef>, VmError> {
    let mut refs = HashMap::new();

    for (zero_based_index, constant) in class_file.constant_pool.constants.iter().enumerate() {
        if let Constant::Methodref {
            class_index,
            name_and_type_index,
        }
        | Constant::InterfaceMethodref {
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

fn collect_string_refs(class_file: &ClassFile) -> Result<HashMap<CpIndex, String>, VmError> {
    let mut refs = HashMap::new();

    for (zero_based_index, constant) in class_file.constant_pool.constants.iter().enumerate() {
        if let Constant::String { string_index } = constant {
            refs.insert(
                CpIndex((zero_based_index + 1) as u16),
                class_file.constant_pool.utf8(CpIndex(*string_index))?,
            );
        }
    }

    Ok(refs)
}

fn default_value(ty: &JavaType) -> Value {
    match ty {
        JavaType::Byte | JavaType::Char | JavaType::Int | JavaType::Short | JavaType::Boolean => {
            Value::Int(0)
        }
        JavaType::Long => Value::Long(0),
        JavaType::Float => Value::Float(0.0),
        JavaType::Double => Value::Double(0.0),
        JavaType::Object(_) | JavaType::Array(_) => Value::null(),
        JavaType::Void => Value::Int(0),
    }
}
