use std::collections::HashMap;

use crate::vm::{
    class::RuntimeClass,
    ids::{ClassId, ObjectRef, ThreadId},
    thread::VmThread,
    value::Value,
};

#[derive(Debug, Clone, Default, PartialEq)]
pub struct MethodArea {
    pub classes: Vec<RuntimeClass>,
    pub class_by_name: HashMap<String, ClassId>,
}

impl MethodArea {
    pub fn register_class(&mut self, name: impl Into<String>) -> ClassId {
        let name = name.into();
        if let Some(class_id) = self.class_by_name.get(&name) {
            return *class_id;
        }

        let id = ClassId(self.classes.len());
        self.classes.push(RuntimeClass {
            id,
            name: name.clone(),
            fields: Vec::new(),
            static_fields: Vec::new(),
            class_refs: HashMap::new(),
            field_refs: HashMap::new(),
            methods: Vec::new(),
            method_refs: HashMap::new(),
            string_refs: HashMap::new(),
            state: crate::vm::class::ClassState::Loaded,
        });
        self.class_by_name.insert(name, id);
        id
    }

    pub fn insert_class(&mut self, class: RuntimeClass) -> ClassId {
        if let Some(class_id) = self.class_by_name.get(&class.name) {
            return *class_id;
        }

        let id = class.id;
        self.class_by_name.insert(class.name.clone(), id);
        self.classes.push(class);
        id
    }

    pub fn class(&self, id: ClassId) -> Option<&RuntimeClass> {
        self.classes.get(id.0)
    }

    pub fn class_mut(&mut self, id: ClassId) -> Option<&mut RuntimeClass> {
        self.classes.get_mut(id.0)
    }

    pub fn class_id(&self, name: &str) -> Option<ClassId> {
        self.class_by_name.get(name).copied()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum HeapEntry {
    Object(Object),
    Array(Array),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Object {
    pub class_id: ClassId,
    pub fields: Vec<Value>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Array {
    Int(Vec<i32>),
    Ref(Vec<Option<ObjectRef>>),
    Values(Vec<Value>),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Heap {
    objects: Vec<HeapEntry>,
}

impl Heap {
    pub fn allocate_object(&mut self, class_id: ClassId, fields: Vec<Value>) -> ObjectRef {
        let reference = ObjectRef(self.objects.len());
        self.objects
            .push(HeapEntry::Object(Object { class_id, fields }));
        reference
    }

    pub fn allocate_array(&mut self, array: Array) -> ObjectRef {
        let reference = ObjectRef(self.objects.len());
        self.objects.push(HeapEntry::Array(array));
        reference
    }

    pub fn get(&self, reference: ObjectRef) -> Option<&HeapEntry> {
        self.objects.get(reference.0)
    }

    pub fn get_mut(&mut self, reference: ObjectRef) -> Option<&mut HeapEntry> {
        self.objects.get_mut(reference.0)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct VirtualMemory {
    pub method_area: MethodArea,
    pub heap: Heap,
    pub threads: Vec<VmThread>,
    pub stdout: Vec<String>,
    pub stderr: Vec<String>,
}

impl VirtualMemory {
    pub fn create_thread(&mut self) -> ThreadId {
        let id = ThreadId(self.threads.len());
        self.threads.push(VmThread::new(id));
        id
    }

    pub fn thread(&self, id: ThreadId) -> Option<&VmThread> {
        self.threads.get(id.0)
    }

    pub fn thread_mut(&mut self, id: ThreadId) -> Option<&mut VmThread> {
        self.threads.get_mut(id.0)
    }
}
