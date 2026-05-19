use crate::{
    structure::class::ClassFile,
    vm::{
        class::RuntimeClass,
        cpu::VirtualCpu,
        error::{UnsupportedFeature, VmError},
        frame::Frame,
        ids::ClassId,
        loader::{ClassLoader, ClassSource},
        memory::{Array, VirtualMemory},
        value::Value,
    },
};

#[derive(Debug, Default)]
pub struct Vm {
    pub memory: VirtualMemory,
}

impl Vm {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn load_class_file(&mut self, class_file: &ClassFile) -> Result<ClassId, VmError> {
        let id = ClassId(self.memory.method_area.classes.len());
        let runtime_class = RuntimeClass::from_class_file(id, class_file)?;
        Ok(self.memory.method_area.insert_class(runtime_class))
    }

    pub fn load_class_files<'a>(
        &mut self,
        class_files: impl IntoIterator<Item = &'a ClassFile>,
    ) -> Result<Vec<ClassId>, VmError> {
        class_files
            .into_iter()
            .map(|class_file| self.load_class_file(class_file))
            .collect()
    }

    pub fn load_class_from_source<S: ClassSource>(
        &mut self,
        loader: &mut ClassLoader<S>,
        binary_name: &str,
    ) -> Result<ClassId, VmError> {
        loader.load_class(binary_name, &mut self.memory)
    }

    pub fn invoke_static(
        &mut self,
        class_name: &str,
        method_name: &str,
        descriptor: &str,
        args: Vec<Value>,
    ) -> Result<Option<Value>, VmError> {
        let class_id = self
            .memory
            .method_area
            .class_id(class_name)
            .ok_or_else(|| VmError::ClassNotFound(class_name.to_string()))?;
        let class = self
            .memory
            .method_area
            .class(class_id)
            .ok_or_else(|| VmError::ClassNotFound(class_name.to_string()))?;
        let method =
            class
                .find_method(method_name, descriptor)
                .ok_or_else(|| VmError::NoSuchMethod {
                    class: class_name.to_string(),
                    name: method_name.to_string(),
                    descriptor: descriptor.to_string(),
                })?;
        let code = method.code.clone().ok_or_else(|| {
            VmError::UnsupportedFeature(UnsupportedFeature::NativeMethod {
                class: class_name.to_string(),
                name: method_name.to_string(),
                descriptor: descriptor.to_string(),
            })
        })?;
        let method_id = method.id;
        let max_locals = method.max_locals as usize;

        let thread_id = self.memory.create_thread();
        let mut frame = Frame::new(class_id, method_id, max_locals, code);
        for (index, value) in args.into_iter().enumerate() {
            let local = frame.locals.get_mut(index).ok_or_else(|| {
                VmError::VerificationError(format!("local {index} does not exist"))
            })?;
            *local = value;
        }

        self.memory
            .thread_mut(thread_id)
            .ok_or_else(|| {
                VmError::InternalError(format!("thread {:?} was not created", thread_id))
            })?
            .push_frame(frame);

        let mut cpu = VirtualCpu::new(thread_id);
        cpu.run_until_return(&mut self.memory)
    }

    pub fn invoke_public_static_main(
        &mut self,
        class_name: &str,
    ) -> Result<Option<Value>, VmError> {
        let args_ref = self.memory.heap.allocate_array(Array::Ref(Vec::new()));
        self.invoke_static(
            class_name,
            "main",
            "([Ljava/lang/String;)V",
            vec![Value::Ref(Some(args_ref))],
        )
    }
}
