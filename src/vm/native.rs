use crate::vm::{
    class::RuntimeMethodRef,
    descriptor::parse_method_descriptor,
    error::{UnsupportedFeature, VmError},
    memory::VirtualMemory,
    value::Value,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeInvokeResult {
    Handled,
    Unsupported,
}

pub struct NativeMethods;

impl NativeMethods {
    pub fn virtual_parameter_count(
        method_ref: &RuntimeMethodRef,
    ) -> Result<Option<usize>, VmError> {
        if !is_print_stream_output(method_ref) {
            return Ok(None);
        }

        let descriptor = parse_method_descriptor(&method_ref.descriptor)?;
        Ok(Some(descriptor.parameters.len()))
    }

    pub fn invoke_virtual(
        memory: &mut VirtualMemory,
        method_ref: &RuntimeMethodRef,
        receiver: Value,
        args: Vec<Value>,
    ) -> Result<NativeInvokeResult, VmError> {
        if !is_print_stream_output(method_ref) {
            return Ok(NativeInvokeResult::Unsupported);
        }

        let is_stderr = match receiver {
            Value::NativeStdout => false,
            Value::NativeStderr => true,
            value => {
                return Err(VmError::VerificationError(format!(
                    "print/println expected PrintStream receiver, found {value:?}"
                )));
            }
        };
        let text = if let Some(value) = args.first() {
            printable_value(value, method_ref)?
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

        Ok(NativeInvokeResult::Handled)
    }
}

fn is_print_stream_output(method_ref: &RuntimeMethodRef) -> bool {
    method_ref.class_name == "java/io/PrintStream"
        && (method_ref.name == "println" || method_ref.name == "print")
}

fn printable_value(value: &Value, method_ref: &RuntimeMethodRef) -> Result<String, VmError> {
    match value {
        Value::Int(value) => Ok(value.to_string()),
        Value::Long(value) => Ok(value.to_string()),
        Value::Float(value) => Ok(value.to_string()),
        Value::Double(value) => Ok(value.to_string()),
        Value::String(value) => Ok(value.clone()),
        Value::Ref(None) => Ok("null".to_string()),
        value => Err(VmError::UnsupportedFeature(
            UnsupportedFeature::NativeMethod {
                class: method_ref.class_name.clone(),
                name: method_ref.name.clone(),
                descriptor: format!("{value:?}"),
            },
        )),
    }
}
