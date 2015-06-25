use std::collections::LinkedList;
use super::{Node, TestNode, TestState};
use super::TestState::*;
use core::Port::*;
use image::Image;
use io::IoBusView;

#[derive(Debug)]
pub struct TestInputNode {
    test_data: LinkedList<isize>,
    blocked: bool,
}

impl TestInputNode {
    pub fn with_data(test_data: &Vec<isize>) -> TestInputNode {
        TestInputNode {
            test_data: test_data.iter().map(|&i| i).collect::<LinkedList<_>>(),
            blocked: false,
        }
    }
}

impl Node for TestInputNode {
    fn step(&mut self, io: &mut IoBusView) {
        if !self.blocked {
            if let Some(&val) = self.test_data.front() {
                io.write(DOWN, val);
                self.blocked = true;
            }
        }
    }

    fn sync(&mut self, io: &mut IoBusView) {
        if !io.is_blocked() {
            self.test_data.pop_front();
            self.blocked = false;
        }
    }
}

#[derive(Debug)]
pub struct TestOutputNode {
    test_data: LinkedList<isize>,
    results: Vec<(isize, isize)>,
}

impl TestOutputNode {
    pub fn with_data(test_data: &Vec<isize>) -> TestOutputNode {
        TestOutputNode {
            test_data: test_data.iter().map(|&i| i).collect::<LinkedList<_>>(),
            results: Vec::new(),
        }
    }
}

impl Node for TestOutputNode {
    fn step(&mut self, io: &mut IoBusView) {
        if let Some(val) = io.read(UP) {
            if let Some(expected) = self.test_data.pop_front() {
                self.results.push((expected, val));
            }
        }
    }
}

impl TestNode for TestOutputNode {
    fn state(&self) -> TestState {
        if !self.test_data.is_empty() {
            Testing
        } else {
            if self.results.iter().all(|&(e, a)| e == a) {
                Passed
            } else {
                Failed
            }
        }
    }
}

#[derive(Debug)]
pub struct TestImageNode {
    test_image: Image,
    image: Image,
}

impl TestImageNode {
    pub fn with_data(data: &Vec<isize>, width: usize, height: usize) -> TestImageNode {
        TestImageNode {
            test_image: Image::with_data(data, width, height),
            image: Image::new(width, height),
        }
    }
}

impl Node for TestImageNode {
    fn step(&mut self, io: &mut IoBusView) {
        if let Some(val) = io.read(UP) {
            self.image.write(val);
        }
    }
}

impl TestNode for TestImageNode {
    fn state(&self) -> TestState {
        if self.test_image == self.image {
            Passed
        } else {
            Testing
        }
    }
}
