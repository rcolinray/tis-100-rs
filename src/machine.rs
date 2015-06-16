//! TIS-100 emulator implementations.

use core::Port::*;
use io::IoBus;
use node::{Node, BasicExecutionNode};
use save::Save;

/// Implements the *Simple Sandbox* puzzle from the game.
///
/// Example:
///
/// ```
/// use tis_100::save::parse_save;
/// use tis_100::machine::Sandbox;
///
/// // This program reads the value from the console and simply passes it to the console output.
/// let src = "@1\nMOV UP DOWN\n@5\nMOV UP DOWN\n@9\nMOV UP RIGHT\n@10\nMOV LEFT DOWN\n";
///
/// let save = parse_save(src).unwrap();
/// let mut sandbox = Sandbox::with_save(&save);
///
/// sandbox.write_console(42);
///
/// for _ in 0..5 {
///     sandbox.step();
/// }
///
/// assert_eq!(sandbox.read_console(), Some(42));
/// ```
#[derive(Debug)]
pub struct Sandbox {
    nodes: Vec<BasicExecutionNode>,
    bus: IoBus,
}

impl Sandbox {
    /// Construct a new `Sandbox` with programs from the `Save`.
    pub fn with_save(save: &Save) -> Sandbox {
        let mut sandbox = Sandbox {
            nodes: Vec::new(),
            bus: IoBus::new(),
        };
        sandbox.setup(save);
        sandbox
    }

    /// Setup the connections between nodes. Each node is fully connected to its neighbors.
    fn setup(&mut self, save: &Save) {
        for node_num in 0..12 {
            match save.get(&node_num) {
                Some(prog) => self.nodes.push(BasicExecutionNode::with_program(prog.clone())),
                None => self.nodes.push(BasicExecutionNode::new()),
            };
        }

        self.bus.connect_full(0, 1, RIGHT)
            .connect_full(1, 2, RIGHT)
            .connect_full(2, 3, RIGHT)
            .connect_full(4, 5, RIGHT)
            .connect_full(5, 6, RIGHT)
            .connect_full(6, 7, RIGHT)
            .connect_full(8, 9, RIGHT)
            .connect_full(9, 10, RIGHT)
            .connect_full(10, 11, RIGHT);

        self.bus.connect_full(0, 4, DOWN)
            .connect_full(1, 5, DOWN)
            .connect_full(2, 6, DOWN)
            .connect_full(3, 7, DOWN)
            .connect_full(4, 8, DOWN)
            .connect_full(5, 9, DOWN)
            .connect_full(6, 10, DOWN)
            .connect_full(7, 11, DOWN)
            // The console output is connected to node 10.
            .connect_half(10, 12, DOWN)
            // The console input is connected to node 1.
            .connect_half(12, 1, DOWN);
    }

    /// Step each node through one instruction.
    pub fn step(&mut self) {
        // Step each node
        for (id, node) in self.nodes.iter_mut().enumerate() {
            let mut view = self.bus.view(id);
            node.step(&mut view);
        }

        // Synchronize writes and reads on each node
        for (id, node) in self.nodes.iter_mut().enumerate() {
            let mut view = self.bus.view(id);
            node.sync(&mut view);
        }

        // Commit writes so they are available on the next cycle.
        self.bus.commit();
    }

    /// Send a value from the console to node 1.
    pub fn write_console(&mut self, value: isize) {
        self.bus.view(12).write(DOWN, value);
    }

    /// Read a value from node 10 and send it to the console.
    pub fn read_console(&mut self) -> Option<isize> {
        self.bus.view(12).read(UP)
    }
}
