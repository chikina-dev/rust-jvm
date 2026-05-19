use crate::vm::{frame::Frame, ids::ThreadId};

#[derive(Debug, Clone, PartialEq)]
pub struct VmThread {
    pub id: ThreadId,
    pub frames: Vec<Frame>,
}

impl VmThread {
    pub fn new(id: ThreadId) -> Self {
        Self {
            id,
            frames: Vec::new(),
        }
    }

    pub fn push_frame(&mut self, frame: Frame) {
        self.frames.push(frame);
    }

    pub fn pop_frame(&mut self) -> Option<Frame> {
        self.frames.pop()
    }

    pub fn current_frame(&self) -> Option<&Frame> {
        self.frames.last()
    }

    pub fn current_frame_mut(&mut self) -> Option<&mut Frame> {
        self.frames.last_mut()
    }
}
