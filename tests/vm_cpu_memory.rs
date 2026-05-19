use rust_jvm::vm::{
    cpu::{CpuState, StepResult, VirtualCpu},
    error::{UnsupportedFeature, VmError},
    frame::Frame,
    ids::MethodId,
    instruction::{DecodedInstruction, DecodedInstructionKind},
    memory::{Array, HeapEntry, VirtualMemory},
    value::Value,
};

fn instruction(
    pc: u32,
    next_pc: u32,
    opcode: u8,
    mnemonic: &'static str,
    kind: DecodedInstructionKind,
) -> DecodedInstruction {
    DecodedInstruction {
        pc,
        next_pc,
        opcode,
        mnemonic,
        kind,
    }
}

fn push_frame(memory: &mut VirtualMemory, frame: Frame) -> VirtualCpu {
    let thread_id = memory.create_thread();
    memory
        .thread_mut(thread_id)
        .expect("thread should exist")
        .push_frame(frame);
    VirtualCpu::new(thread_id)
}

#[test]
fn virtual_memory_registers_classes_and_allocates_heap_entries() {
    let mut memory = VirtualMemory::default();

    let class_id = memory.method_area.register_class("Example");
    let same_class_id = memory.method_area.register_class("Example");
    let object_ref = memory.heap.allocate_object(class_id, vec![Value::Int(10)]);
    let array_ref = memory.heap.allocate_array(Array::Int(vec![1, 2, 3]));

    assert_eq!(class_id, same_class_id);
    assert_eq!(memory.method_area.class_id("Example"), Some(class_id));
    assert_eq!(
        memory
            .method_area
            .class(class_id)
            .map(|class| class.name.as_str()),
        Some("Example")
    );
    assert!(matches!(
        memory.heap.get(object_ref),
        Some(HeapEntry::Object(object)) if object.fields == vec![Value::Int(10)]
    ));
    assert!(matches!(
        memory.heap.get(array_ref),
        Some(HeapEntry::Array(Array::Int(values))) if values == &vec![1, 2, 3]
    ));
}

#[test]
fn virtual_cpu_executes_int_stack_and_local_flow_until_return() {
    let mut memory = VirtualMemory::default();
    let class_id = memory.method_area.register_class("Example");
    let frame = Frame::new(
        class_id,
        MethodId(0),
        1,
        vec![
            instruction(0, 1, 0x04, "iconst_1", DecodedInstructionKind::IConst(1)),
            instruction(1, 2, 0x3b, "istore_0", DecodedInstructionKind::IStore(0)),
            instruction(2, 3, 0x05, "iconst_2", DecodedInstructionKind::IConst(2)),
            instruction(3, 4, 0x1a, "iload_0", DecodedInstructionKind::ILoad(0)),
            instruction(4, 5, 0x60, "iadd", DecodedInstructionKind::IAdd),
            instruction(5, 6, 0xac, "ireturn", DecodedInstructionKind::IReturn),
        ],
    );
    let mut cpu = push_frame(&mut memory, frame);

    assert_eq!(cpu.run_until_return(&mut memory), Ok(Some(Value::Int(3))));
    assert_eq!(cpu.state, CpuState::Halted);
}

#[test]
fn virtual_cpu_step_updates_pc_for_conditional_branch_target() {
    let mut memory = VirtualMemory::default();
    let class_id = memory.method_area.register_class("Branch");
    let frame = Frame::new(
        class_id,
        MethodId(0),
        0,
        vec![
            instruction(0, 1, 0x03, "iconst_0", DecodedInstructionKind::IConst(0)),
            instruction(1, 4, 0x99, "ifeq", DecodedInstructionKind::IfEq(4)),
            instruction(4, 5, 0x04, "iconst_1", DecodedInstructionKind::IConst(1)),
            instruction(5, 6, 0x05, "iconst_2", DecodedInstructionKind::IConst(2)),
        ],
    );
    let mut cpu = push_frame(&mut memory, frame);

    assert_eq!(cpu.step(&mut memory), Ok(StepResult::Continue));
    assert_eq!(cpu.step(&mut memory), Ok(StepResult::Continue));

    let frame = memory
        .thread(cpu.current_thread)
        .and_then(|thread| thread.current_frame())
        .expect("current frame should exist");
    assert_eq!(frame.pc, 5);
}

#[test]
fn virtual_cpu_returns_unsupported_feature_for_known_unimplemented_opcode() {
    let mut memory = VirtualMemory::default();
    let class_id = memory.method_area.register_class("Unsupported");
    let frame = Frame::new(
        class_id,
        MethodId(0),
        0,
        vec![instruction(
            0,
            5,
            0xba,
            "invokedynamic",
            DecodedInstructionKind::Unsupported {
                mnemonic: "invokedynamic",
            },
        )],
    );
    let mut cpu = push_frame(&mut memory, frame);

    assert_eq!(
        cpu.step(&mut memory),
        Err(VmError::UnsupportedFeature(UnsupportedFeature::Opcode {
            opcode: 0xba,
            mnemonic: "invokedynamic",
        }))
    );
}
