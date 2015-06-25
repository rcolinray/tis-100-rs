//! Types of nodes used in the TIS-100.

pub use self::exec::{BasicExecutionNode, DamagedExecutionNode};
pub use self::stack::StackMemoryNode;
pub use self::test::{TestInputNode, TestOutputNode, TestImageNode};

mod exec;
mod stack;
mod test;

use io::IoBusView;

/// Interface for nodes in a TIS-100 system.
pub trait Node {
    /// Execute a single instruction cycle.
    #[allow(unused)]
    fn step(&mut self, io: &mut IoBusView) {

    }

    /// Synchronize reads and writes after the last instruction cycle.
    #[allow(unused)]
    fn sync(&mut self, io: &mut IoBusView) {

    }

    /// Determine if a node is executing assembly code or if it is stalled on a read or write.
    /// For all nodes except nodes which can execute assembly, this should always be `true`.
    fn is_stalled(&self) -> bool {
        true
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum TestState {
    Testing,
    Passed,
    Failed,
}

pub trait TestNode: Node {
    fn state(&self) -> TestState;
}
