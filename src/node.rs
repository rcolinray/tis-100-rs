use core::{Port, Register, Source, Instruction, Program};
use core::Instruction::*;
use core::Port::*;
use core::IoRegister::*;
use core::Register::*;
use core::Source::*;
use io::IoBusView;

/// Interface for nodes in a TIS-100 system.
pub trait Node {
    fn step(&mut self, io: &mut IoBusView);
    fn sync(&mut self, io: &mut IoBusView);
}

/// A corrupted TIS-100 node. `step` and `sync` have no effect.
#[derive(Debug)]
pub struct CorruptedNode;

impl Node for CorruptedNode {
    #[allow(unused)]
    fn step(&mut self, io: &mut IoBusView) {

    }

    #[allow(unused)]
    fn sync(&mut self, io: &mut IoBusView) {

    }
}

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
        let dirs = vec![UP, DOWN, LEFT, RIGHT];

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

/// An execution mode of a `BasicExecutionNode`.
#[derive(Debug, PartialEq, Eq)]
pub enum Mode {
    Idle,
    Run,
    Read,
    Wrte,
}

use self::Mode::*;

/// Executes TIS-100 assembly code. Once a `Program` has been set on the node, the node may be
/// executed using alternating calls of `step` and `sync`.
///
/// Example:
///
/// ```
/// use tis_100::core::Port::*;
/// use tis_100::io::IoBus;
/// use tis_100::node::{Node, BasicExecutionNode};
/// use tis_100::parse::parse_program;
///
/// let src = "MOV UP ACC\nADD 1\nMOV ACC DOWN\n";
/// let prog = parse_program(src).unwrap();
/// let mut bus = IoBus::new();
/// let mut node = BasicExecutionNode::with_program(prog);
///
/// bus.connect_half(0, 1, DOWN)
///     .connect_half(1, 2, DOWN)
///     .view(0).write(DOWN, 1);
/// bus.commit();
///
/// for _ in 0..3 {
///     {
///         let mut view = bus.view(1);
///         node.step(&mut view);
///         node.sync(&mut view);
///     }
///
///     bus.commit();
/// }
///
/// assert_eq!(bus.view(2).read(UP), Some(2));
/// ```
#[derive(Debug)]
pub struct BasicExecutionNode {
    program: Program,
    pc: isize,
    mode: Mode,
    acc: isize,
    bak: isize,
    last: Option<Port>,
}

impl BasicExecutionNode {
    /// Construct a new, empty `BasicExecutionNode`.
    pub fn new() -> BasicExecutionNode {
        BasicExecutionNode {
            program: Program::new(),
            pc: 0,
            mode: Idle,
            acc: 0,
            bak: 0,
            last: None,
        }
    }

    /// Construct a new `BasicExecutionNode` and initialize it with the given program.
    pub fn with_program(program: Program) -> BasicExecutionNode {
        let mut node = BasicExecutionNode::new();
        node.set_program(program);
        node
    }

    /// Set the program on a `BasicExecutionNode`.
    pub fn set_program(&mut self, program: Program) {
        self.program = program;
    }

    pub fn get_mode(&self) -> &Mode {
        &self.mode
    }

    /// Increment the program counter.
    fn inc_pc(&mut self) {
        self.pc += 1;
        if self.pc >= self.program.len() as isize {
            self.pc = 0;
        }
    }

    /// Set the value of the program counter.
    fn set_pc(&mut self, pc: isize) {
        if pc < 0 {
            self.pc = 0;
        } else if pc as usize > self.program.len() {
            self.pc = self.program.len() as isize - 1;
        } else {
            self.pc = pc;
        }
    }

    /// Fetch the instruction at the current program counter.
    fn fetch(&mut self) -> Option<Instruction> {
        self.program.get(self.pc as usize).map(|&i| i)
    }

    /// Evaluate the given instruction.
    fn eval(&mut self, instruction: Instruction, io: &mut IoBusView) {
        match instruction {
            Nop => (),
            Mov(src, dst) => if let Some(val) = self.read(io, src) {
                let value = clamp_value(val);
                self.write(io, dst, value);
            },
            Swp => {
                let tmp = self.bak;
                self.bak = self.acc;
                self.acc = tmp;
            },
            Sav => self.bak = self.acc,
            Add(src) => if let Some(val) = self.read(io, src) {
                self.acc += val;
            },
            Sub(src) => if let Some(val) = self.read(io, src) {
                self.acc -= val;
            },
            Neg => self.acc = -self.acc,
            Jmp(pc) => self.set_pc(pc),
            Jez(pc) => if self.acc == 0 {
                self.set_pc(pc);
            },
            Jnz(pc) => if self.acc != 0 {
                self.set_pc(pc);
            },
            Jgz(pc) => if self.acc > 0 {
                self.set_pc(pc);
            },
            Jlz(pc) => if self.acc < 0 {
                self.set_pc(pc);
            },
            Jro(src) => if let Some(off) = self.read(io, src) {
                let pc = self.pc + off;
                self.set_pc(pc);
            },
        }
    }

    /// Read a value from the given register.
    fn read(&mut self, io: &mut IoBusView, src: Source) -> Option<isize> {
        let val = match src {
            VAL(val) => Some(val),
            REG(ACC) => Some(self.acc),
            REG(NIL) => Some(0),
            REG(IO(DIR(port))) => io.read(port),
            REG(IO(ANY)) => io.read(UP)
                .or_else(|| io.read(DOWN))
                .or_else(|| io.read(LEFT))
                .or_else(|| io.read(RIGHT)),
            REG(IO(LAST)) => match self.last {
                Some(port) => io.read(port),
                None => Some(0),
            },
        };

        val.or_else(|| {
            self.mode = Read;
            None
        })
    }

    /// Write a value to the given register.
    fn write(&mut self, io: &mut IoBusView, dst: Register, value: isize) {
        match dst {
            ACC => self.acc = value,
            NIL => (),
            IO(reg) => {
                match reg {
                    DIR(port) => io.write(port, value),
                    ANY => {
                        io.write(UP, value);
                        io.write(DOWN, value);
                        io.write(LEFT, value);
                        io.write(RIGHT, value);
                    },
                    LAST => if let Some(port) = self.last {
                        io.write(port, value);
                    }
                }
                self.mode = Wrte;
            }
        }
    }
}

impl Node for BasicExecutionNode {
    /// Execute the next instruction, if possible.
    fn step(&mut self, io: &mut IoBusView) {
        if self.mode != Wrte {
            if let Some(instruction) = self.fetch() {
                self.mode = Run;
                self.eval(instruction, io);
                if self.mode == Run {
                    self.inc_pc();
                }
            }
        }
    }

    /// Synchronize this node with the `IoBus`. If the node was blocked on a write, and that value
    /// was read during the previous cycle, then this will clear the block and allow the node to
    /// proceed with execution.
    fn sync(&mut self, io: &mut IoBusView) {
        if self.mode == Wrte {
            if !io.is_blocked() {
                self.mode = Run;
                self.inc_pc();
            }
        }
    }
}

/// Limit a value in a TIS-100 register to the range -999..999 inclusive.
fn clamp_value(value: isize) -> isize {
    if value > 999 {
        999
    } else if value < -999 {
        -999
    } else {
        value
    }
}

#[test]
fn test_clamp_value() {
    assert_eq!(clamp_value(1000), 999);
    assert_eq!(clamp_value(999), 999);
    assert_eq!(clamp_value(998), 998);

    assert_eq!(clamp_value(-1000), -999);
    assert_eq!(clamp_value(-999), -999);
    assert_eq!(clamp_value(-998), -998);
}
