use crate::vm::{
    error::VmError,
    loader::{ClassLoader, InMemoryClassSource},
    value::Value,
    vm::Vm,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassBytes {
    pub binary_name: String,
    pub bytes: Vec<u8>,
}

impl ClassBytes {
    pub fn new(binary_name: impl Into<String>, bytes: impl Into<Vec<u8>>) -> Self {
        Self {
            binary_name: binary_name.into(),
            bytes: bytes.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VmExecutionResult {
    pub return_value: Option<Value>,
    pub stdout: Vec<String>,
    pub stderr: Vec<String>,
}

pub fn invoke_static_from_class_bytes(
    classes: impl IntoIterator<Item = ClassBytes>,
    root_class_name: &str,
    method_name: &str,
    descriptor: &str,
    args: Vec<Value>,
) -> Result<VmExecutionResult, VmError> {
    let mut vm = vm_from_class_bytes(classes, root_class_name)?;
    let return_value = vm.invoke_static(root_class_name, method_name, descriptor, args)?;
    Ok(VmExecutionResult::from_vm(return_value, &vm))
}

pub fn invoke_public_static_main_from_class_bytes(
    classes: impl IntoIterator<Item = ClassBytes>,
    root_class_name: &str,
) -> Result<VmExecutionResult, VmError> {
    let mut vm = vm_from_class_bytes(classes, root_class_name)?;
    let return_value = vm.invoke_public_static_main(root_class_name)?;
    Ok(VmExecutionResult::from_vm(return_value, &vm))
}

fn vm_from_class_bytes(
    classes: impl IntoIterator<Item = ClassBytes>,
    root_class_name: &str,
) -> Result<Vm, VmError> {
    let mut source = InMemoryClassSource::new();
    for class in classes {
        source.insert_class_bytes(class.binary_name, class.bytes);
    }

    let mut loader = ClassLoader::new(source);
    let mut vm = Vm::new();
    loader.load_available_class_closure(root_class_name, &mut vm.memory)?;
    Ok(vm)
}

impl VmExecutionResult {
    fn from_vm(return_value: Option<Value>, vm: &Vm) -> Self {
        Self {
            return_value,
            stdout: vm.memory.stdout.clone(),
            stderr: vm.memory.stderr.clone(),
        }
    }
}
