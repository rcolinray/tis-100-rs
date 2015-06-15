use core::Port::*;
use io::IoBus;
use node::{Node, BasicExecutionNode};
use save::Save;

#[derive(Debug)]
pub struct Tis100 {
    nodes: Vec<BasicExecutionNode>,
    bus: IoBus,
}

impl Tis100 {
    pub fn with_save(save: &Save) -> Tis100 {
        let mut tis100 = Tis100 {
            nodes: Vec::new(),
            bus: IoBus::new(),
        };
        tis100.setup(save);
        tis100
    }

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
            .connect_half(10, 12, DOWN)
            .connect_half(12, 1, DOWN);
    }

    pub fn step(&mut self) {
        for (id, node) in self.nodes.iter_mut().enumerate() {
            let mut view = self.bus.view(id);
            node.step(&mut view);
        }

        for (id, node) in self.nodes.iter_mut().enumerate() {
            let mut view = self.bus.view(id);
            node.sync(&mut view);
        }

        self.bus.commit();
    }

    pub fn write_console(&mut self, value: isize) {
        self.bus.view(12).write(DOWN, value);
    }

    pub fn read_console(&mut self) -> Option<isize> {
        self.bus.view(12).read(UP)
    }
}
