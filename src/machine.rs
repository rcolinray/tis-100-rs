//! TIS-100 emulator implementations.

use std::collections::VecMap;
use core::Port::*;
use io::IoBus;
use node::{Node, TestNode, TestState, BasicExecutionNode};
use node::TestState::*;
use save::Save;
use spec::Spec;

pub const NUM_NODES: usize = 12;

pub const NUM_INPUTS: usize = 4;
pub const INPUT_0: usize = 12;
pub const INPUT_1: usize = 13;
pub const INPUT_2: usize = 14;
pub const INPUT_3: usize = 15;

pub const NUM_OUTPUTS: usize = 4;
pub const OUTPUT_0: usize = 16;
pub const OUTPUT_1: usize = 17;
pub const OUTPUT_2: usize = 18;
pub const OUTPUT_3: usize = 19;

/// Implements the *Simple Sandbox* puzzle from the game.
///
/// # Example
///
/// ```
/// use tis_100::save::parse_save;
/// use tis_100::machine::Sandbox;
///
/// // This program reads the value from the console and simply passes it to the console output.
/// let src = "@1\nMOV UP DOWN\n@5\nMOV UP DOWN\n@9\nMOV UP RIGHT\n@10\nMOV LEFT DOWN\n";
///
/// let save = parse_save(src).unwrap();
/// let mut sandbox = Sandbox::from_save(&save);
///
/// sandbox.write_console(42);
///
/// for _ in 0..5 {
///     sandbox.step();
/// }
///
/// assert_eq!(sandbox.read_console(), Some(42));
/// ```
pub struct Sandbox {
    cpu: Tis100,
}

impl Sandbox {
    /// Construct a new `Sandbox` with programs from the `Save`.
    pub fn from_save(save: &Save) -> Sandbox {
        let mut sandbox = Sandbox {
            cpu: Tis100::new(),
        };
        sandbox.setup(save);
        sandbox
    }

    /// Setup the connections between nodes. Each node is fully connected to its neighbors.
    fn setup(&mut self, save: &Save) {
        for node_num in 0..NUM_NODES {
            match save.get(&node_num) {
                Some(prog) => self.cpu.add_node(node_num, Box::new(BasicExecutionNode::with_program(prog.clone()))),
                None => self.cpu.add_node(node_num, Box::new(BasicExecutionNode::new())),
            };
        }
    }

    /// Step each node through one instruction.
    pub fn step(&mut self) {
        self.cpu.step();
        self.cpu.sync();
        self.cpu.commit();
    }

    /// Write a value to the console.
    pub fn write_console(&mut self, value: isize) {
        self.cpu.write_input(1, value);
    }

    /// Read a value from the console.
    pub fn read_console(&mut self) -> Option<isize> {
        self.cpu.read_output(2)
    }
}

/// Executes arbitrary puzzles using a spec file.
pub struct Puzzle {
    cpu: Tis100,
    tests: VecMap<Box<TestNode>>,
    cycles: usize,
}

impl Puzzle {
    pub fn from_spec(spec: &mut Spec) -> Puzzle {
        let mut cpu = Tis100::new();
        spec.setup(&mut cpu);

        let tests = spec.tests();

        Puzzle {
            cpu: cpu,
            tests: tests,
            cycles: 0,
        }
    }

    pub fn step(&mut self) {
        for (id, node) in self.tests.iter_mut() {
            let mut view = self.cpu.bus.view(id + OUTPUT_0);
            node.step(&mut view);
        }

        self.cpu.step();

        for (id, node) in self.tests.iter_mut() {
            let mut view = self.cpu.bus.view(id + OUTPUT_0);
            node.sync(&mut view);
        }

        self.cpu.sync();
        self.cpu.commit();

        self.cycles += 1;
    }

    pub fn state(&self) -> TestState {
        let states = self.tests.iter().map(|(_, n)| n.state()).collect::<Vec<_>>();

        if states.iter().any(|&s| s == Testing) {
            Testing
        } else if states.iter().any(|&s| s == Failed) {
            Failed
        } else {
            Passed
        }
    }

    pub fn is_deadlocked(&self) -> bool {
        self.cpu.is_deadlocked()
    }

    pub fn cycles(&self) -> usize {
        self.cycles
    }
}

/// An empty TIS-100 CPU.
pub struct Tis100 {
    nodes: VecMap<Box<Node>>,
    bus: IoBus,
    stalled: usize,
}

impl Tis100 {
    /// Construct a new, empty `Tis100`.
    pub fn new() -> Tis100 {
        let mut tis100 = Tis100 {
            nodes: VecMap::new(),
            bus: IoBus::new(),
            stalled: 0,
        };
        tis100.setup();
        tis100
    }

    /// Setup the IO connections between nodes.
    fn setup(&mut self) {
        // Setup left-right connections between nodes
        self.bus.connect_full(0, 1, RIGHT)
            .connect_full(1, 2, RIGHT)
            .connect_full(2, 3, RIGHT)
            .connect_full(4, 5, RIGHT)
            .connect_full(5, 6, RIGHT)
            .connect_full(6, 7, RIGHT)
            .connect_full(8, 9, RIGHT)
            .connect_full(9, 10, RIGHT)
            .connect_full(10, 11, RIGHT);

        // Setup up-down connections between nodes
        self.bus.connect_full(0, 4, DOWN)
            .connect_full(1, 5, DOWN)
            .connect_full(2, 6, DOWN)
            .connect_full(3, 7, DOWN)
            .connect_full(4, 8, DOWN)
            .connect_full(5, 9, DOWN)
            .connect_full(6, 10, DOWN)
            .connect_full(7, 11, DOWN);

        // Setup input connections.
        self.bus.connect_half(INPUT_0, 0, DOWN)
            .connect_half(INPUT_1, 1, DOWN)
            .connect_half(INPUT_2, 2, DOWN)
            .connect_half(INPUT_3, 3, DOWN);

        // Setup output connections.
        self.bus.connect_half(8, OUTPUT_0, DOWN)
            .connect_half(9, OUTPUT_1, DOWN)
            .connect_half(10, OUTPUT_2, DOWN)
            .connect_half(11, OUTPUT_3, DOWN);
    }

    /// Add a new node with the given ID to the system.
    pub fn add_node(&mut self, index: usize, node: Box<Node>) {
        self.nodes.insert(index, node);
    }

    /// Write a value to an input.
    pub fn write_input(&mut self, input: usize, value: isize) {
        assert!(input < NUM_INPUTS);
        self.bus.view(input + INPUT_0).write(DOWN, value);
    }

    /// Read a value from an output.
    pub fn read_output(&mut self, output: usize) -> Option<isize> {
        assert!(output < NUM_OUTPUTS);
        self.bus.view(output + OUTPUT_0).read(UP)
    }

    /// Execute one instruction cycle on all nodes in the system.
    pub fn step(&mut self) {
        // Step each node
        for (id, node) in self.nodes.iter_mut() {
            let mut view = self.bus.view(id);
            node.step(&mut view);
        }
    }

    /// Synchronize reads and writes for each node.
    pub fn sync(&mut self) {
        // Synchronize writes and reads on each node
        for (id, node) in self.nodes.iter_mut() {
            let mut view = self.bus.view(id);
            node.sync(&mut view);
        }

        // Check for deadlock
        if self.nodes.iter().all(|(_, ref n)| n.is_stalled()) {
            self.stalled += 1;
        } else {
            self.stalled = 0;
        }

    }

    /// Commit all outstanding writes on the `IoBus`.
    pub fn commit(&mut self) {
        // Commit writes so they are available on the next cycle.
        self.bus.commit();
    }

    /// Determine if the system is deadlocked. The system is considered deadlocked if all
    /// execution nodes are reading or writing for more than 1 cycle.
    pub fn is_deadlocked(&self) -> bool {
        self.stalled > 1
    }
}
