use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use crate::{
    structure::class::ClassFileParser,
    vm::{
        class::RuntimeClass,
        error::{ParseError, VmError},
        ids::ClassId,
        memory::VirtualMemory,
    },
};

pub trait ClassSource {
    fn load_class_bytes(&self, binary_name: &str) -> Result<Vec<u8>, VmError>;
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct InMemoryClassSource {
    classes: HashMap<String, Vec<u8>>,
}

impl InMemoryClassSource {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert_class_bytes(
        &mut self,
        binary_name: impl Into<String>,
        bytes: impl Into<Vec<u8>>,
    ) {
        self.classes.insert(binary_name.into(), bytes.into());
    }
}

impl ClassSource for InMemoryClassSource {
    fn load_class_bytes(&self, binary_name: &str) -> Result<Vec<u8>, VmError> {
        self.classes
            .get(binary_name)
            .cloned()
            .ok_or_else(|| VmError::ClassNotFound(binary_name.to_string()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileSystemClassSource {
    root: PathBuf,
}

impl FileSystemClassSource {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    fn class_path(&self, binary_name: &str) -> PathBuf {
        self.root
            .join(Path::new(binary_name))
            .with_extension("class")
    }
}

impl ClassSource for FileSystemClassSource {
    fn load_class_bytes(&self, binary_name: &str) -> Result<Vec<u8>, VmError> {
        let path = self.class_path(binary_name);
        fs::read(&path)
            .map_err(|error| VmError::ClassNotFound(format!("{} ({})", binary_name, error)))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassLoader<S> {
    source: S,
    loaded: HashMap<String, ClassId>,
}

impl<S> ClassLoader<S> {
    pub fn new(source: S) -> Self {
        Self {
            source,
            loaded: HashMap::new(),
        }
    }

    pub fn into_source(self) -> S {
        self.source
    }
}

impl<S: ClassSource> ClassLoader<S> {
    pub fn load_class(
        &mut self,
        binary_name: &str,
        memory: &mut VirtualMemory,
    ) -> Result<ClassId, VmError> {
        if let Some(class_id) = self.loaded.get(binary_name) {
            return Ok(*class_id);
        }

        if let Some(class_id) = memory.method_area.class_id(binary_name) {
            self.loaded.insert(binary_name.to_string(), class_id);
            return Ok(class_id);
        }

        let bytes = self.source.load_class_bytes(binary_name)?;
        let mut parser = ClassFileParser::new();
        let (remaining, class_file) = parser.parse(&bytes).map_err(|error| {
            VmError::Parse(ParseError::ClassFormat(format!(
                "failed to parse class {binary_name}: {error:?}"
            )))
        })?;
        if !remaining.is_empty() {
            return Err(VmError::Parse(ParseError::ClassFormat(format!(
                "class {binary_name} has {} trailing bytes",
                remaining.len()
            ))));
        }

        let id = ClassId(memory.method_area.classes.len());
        let runtime_class = RuntimeClass::from_class_file(id, &class_file)?;
        let actual_name = runtime_class.name.clone();
        let class_id = memory.method_area.insert_class(runtime_class);

        self.loaded.insert(actual_name, class_id);
        self.loaded.insert(binary_name.to_string(), class_id);
        Ok(class_id)
    }

    pub fn load_classes<I, N>(
        &mut self,
        binary_names: I,
        memory: &mut VirtualMemory,
    ) -> Result<Vec<ClassId>, VmError>
    where
        I: IntoIterator<Item = N>,
        N: AsRef<str>,
    {
        binary_names
            .into_iter()
            .map(|binary_name| self.load_class(binary_name.as_ref(), memory))
            .collect()
    }
}

impl Default for ClassLoader<InMemoryClassSource> {
    fn default() -> Self {
        Self::new(InMemoryClassSource::default())
    }
}
