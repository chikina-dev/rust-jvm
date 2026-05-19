use crate::{
    structure::class::ClassFile,
    vm::{
        class::RuntimeClass,
        cpu::VirtualCpu,
        error::VmError,
        ids::ClassId,
        invocation::ResolvedMethod,
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
        let method = ResolvedMethod::resolve(&self.memory, class_name, method_name, descriptor)?;

        let thread_id = self.memory.create_thread();
        let frame = method.frame_with_args(args)?;

        self.memory
            .thread_mut(thread_id)
            .ok_or_else(|| {
                VmError::InternalError(format!("thread {:?} was not created", thread_id))
            })?
            .push_frame(frame);

        let mut cpu = VirtualCpu::new(thread_id);
        if method_name != "<clinit>" {
            cpu.ensure_class_initialized(&mut self.memory, method.class_id)?;
        }
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
