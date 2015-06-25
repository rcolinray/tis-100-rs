use super::Node;
use io::IoBusView;
use core::Port::*;

/// A node which stores values written to it on a stack. When the node is read from it will pop the
/// top value off of the stack and return it.
#[derive(Debug)]
pub struct StackMemoryNode {
    stack: Vec<isize>,
    read_index: Option<usize>,
}

impl StackMemoryNode {
    /// Construct a new, empty `StackMemoryNode`.
    pub fn new() -> StackMemoryNode {
        StackMemoryNode {
            stack: Vec::new(),
            read_index: None,
        }
    }
}

impl Node for StackMemoryNode {
    /// At the start of each cycle, the top value is made available on all ports. Any values that
    /// have been written to this node are then added to the stack.
    fn step(&mut self, io: &mut IoBusView) {
        let dirs = vec![LEFT, RIGHT, UP, DOWN];

        // Use last instead of pop so that the value is only removed if a node reads it.
        if let Some(&val) = self.stack.last() {
            self.read_index = Some(self.stack.len() - 1);
            for &dir in dirs.iter() {
                io.write(dir, val);
            }
        }

        for &dir in dirs.iter() {
            if let Some(val) = io.read(dir) {
                self.stack.push(val);
            }
        }
    }

    // At the end of each cycle, check if the top value was actually read from and clear it from
    // the stack if it was.
    fn sync(&mut self, io: &mut IoBusView) {
        if !io.is_blocked() {
            if let Some(index) = self.read_index {
                self.stack.remove(index);
                self.read_index = None;
            }
        }
    }
}
