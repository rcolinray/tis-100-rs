//! TIS-100 emulator implementations.

use std::collections::VecMap;
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
    cpu: Tis100,
    write_node: usize,
    read_node: usize,
}

impl Sandbox {
    /// Construct a new `Sandbox` with programs from the `Save`.
    pub fn with_save(save: &Save) -> Sandbox {
        let mut sandbox = Sandbox {
            cpu: Tis100::new(),
            // write_node and read_node will both be set during setup.
            write_node: 0,
            read_node: 0,
        };
        sandbox.setup(save);
        sandbox
    }

    /// Setup the connections between nodes. Each node is fully connected to its neighbors.
    fn setup(&mut self, save: &Save) {
        for node_num in 0..12 {
            match save.get(&node_num) {
                Some(prog) => self.cpu.add_node(node_num, Box::new(BasicExecutionNode::with_program(prog.clone()))),
                None => self.cpu.add_node(node_num, Box::new(BasicExecutionNode::new())),
            };
        }

        self.write_node = self.cpu.add_input(1);
        self.read_node = self.cpu.add_output(10);
    }

    /// Step each node through one instruction.
    pub fn step(&mut self) {
        self.cpu.step();
    }

    /// Send a value from the console to node 1.
    pub fn write_console(&mut self, value: isize) {
        self.cpu.write(self.write_node, value);
    }

    /// Read a value from node 10 and send it to the console.
    pub fn read_console(&mut self) -> Option<isize> {
        self.cpu.read(self.read_node)
    }
}

/// An empty TIS-100 CPU.
#[derive(Debug)]
pub struct Tis100 {
    nodes: VecMap<Box<Node>>,
    // Used to track available node IDs for input and output ports
    next_node: usize,
    bus: IoBus,
}

impl Tis100 {
    /// Construct a new, empty `Tis100`.
    pub fn new() -> Tis100 {
        let mut tis100 = Tis100 {
            nodes: VecMap::new(),
            next_node: 12,
            bus: IoBus::new(),
        };
        tis100.setup();
        tis100
    }

    /// Setup the IO connections between nodes.
    fn setup(&mut self) {
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
            .connect_full(7, 11, DOWN);
    }

    /// Add a new node with the given ID to the system.
    pub fn add_node(&mut self, index: usize, node: Box<Node>) {
        self.nodes.insert(index, node);
    }

    /// Create a new input port to a CPU node. Returns a node ID that can be used to write values on
    /// the port. Only nodes 0 through 3 can receive inputs. All inputs are connected `UP` from the
    /// receiving nodes.
    pub fn add_input(&mut self, node: usize) -> usize {
        assert!(node < 4);
        let input_node = self.next_node;
        self.next_node += 1;
        self.bus.connect_half(input_node, node, DOWN);
        input_node
    }

    /// Create a new output port from a CPU node. Returns a node ID that can be used to read values
    /// from the port. Only nodes 8 through 11 can send outputs. All outputs are connected `DOWN`
    /// from the sending nodes.
    pub fn add_output(&mut self, node: usize) -> usize {
        assert!(node > 7 && node < 12);
        let output_node = self.next_node;
        self.next_node += 1;
        self.bus.connect_half(node, output_node, DOWN);
        output_node
    }

    /// Read a value from an output node.
    pub fn read(&mut self, node: usize) -> Option<isize> {
        self.bus.view(node).read(UP)
    }

    /// Send a value to an input node.
    pub fn write(&mut self, node: usize, value: isize) {
        self.bus.view(node).write(DOWN, value);
    }

    /// Execute one instruction cycle on all nodes in the system.
    pub fn step(&mut self) {
        // Step each node
        for (id, node) in self.nodes.iter_mut() {
            let mut view = self.bus.view(id);
            node.step(&mut view);
        }

        // Synchronize writes and reads on each node
        for (id, node) in self.nodes.iter_mut() {
            let mut view = self.bus.view(id);
            node.sync(&mut view);
        }

        // Commit writes so they are available on the next cycle.
        self.bus.commit();
    }
}
