use std::collections::{HashMap, VecMap};
use std::collections::hash_map::Iter;
use core::{Port, opposite_port};

/// A unique identifier for a node.
pub type NodeId = usize;

/// A unique identifier for a port.
pub type PortId = usize;

/// A connection from one node to another through a port.
#[derive(Debug)]
pub struct Connection(PortId, NodeId);

/// An `IoBus` is used to pass messages between nodes. Nodes are represented by `usize` indices.
/// Nodes must first be connected before they can pass messages. Nodes can be connected using either
/// half-duplex or full-duplex channels.
///
/// Example:
///
/// ```
/// use tis_100::core::Port::*;
/// use tis_100::io::IoBus;
///
/// let mut bus = IoBus::new();
///
/// // In this example, 1 is right from 0, and 0 is left from 1.
/// bus.connect_full(0, 1, RIGHT);
/// assert!(bus.is_connected(0, 1, RIGHT));
/// assert!(bus.is_connected(1, 0, LEFT));
/// ```
///
/// Once nodes are connected, they can pass messages through the IoBus using an `IoBusView`.
/// An `IoBusView` ensures that a node can only read or write the ports that it is connected to.
/// All writes are buffered. In order for a node to read a value that another node has sent it,
/// the writes must be committed.
///
/// Example:
///
/// ```
/// use tis_100::core::Port::*;
/// use tis_100::io::IoBus;
///
/// let mut bus = IoBus::new();
/// bus.connect_half(0, 1, RIGHT);
///
/// {
///     let mut view = bus.view(0);
///     view.write(RIGHT, 42);
/// }
///
/// bus.commit();
///
/// {
///     let mut view = bus.view(1);
///     assert_eq!(view.read(LEFT), Some(42));
/// }
/// ```
#[derive(Debug)]
pub struct IoBus {
    next_index: PortId,
    ports: VecMap<isize>,
    writes: VecMap<isize>,
    write_blocks: VecMap<isize>,
    nodes: VecMap<PortMap>,
}

impl IoBus {
    /// Construct a new, empty `IoBus`.
    pub fn new() -> IoBus {
        IoBus {
            next_index: 0,
            ports: VecMap::new(),
            writes: VecMap::new(),
            write_blocks: VecMap::new(),
            nodes: VecMap::new(),
        }
    }

    /// Create a one-way connection from one node to another in the given direction.
    pub fn connect_half(&mut self, from: NodeId, to: NodeId, port: Port) -> &mut Self {
        let to_port = opposite_port(port);

        if !self.map_exists(&from) {
            self.insert_map(from);
        }

        if !self.map_exists(&to) {
            self.insert_map(to);
        }

        self.nodes.get_mut(&from).unwrap().set_output(port, self.next_index, to);
        self.nodes.get_mut(&to).unwrap().set_input(to_port, self.next_index, from);
        self.next_index += 1;

        self
    }

    /// Create a two-way connection from one node to another in the given direction. The opposite
    /// direction will be used when connecting the second half.
    pub fn connect_full(&mut self, from: NodeId, to: NodeId, port: Port) -> &mut Self {
        self.connect_half(from, to, port)
            .connect_half(to, from, opposite_port(port))
    }

    /// Determine if two nodes are connected in a certain direction.
    pub fn is_connected(&self, from: NodeId, to: NodeId, port: Port) -> bool {
        if let Some(map) = self.nodes.get(&from) {
            if let Some(&Connection(_, to_node)) = map.get_output(port) {
                if let Some(map) = self.nodes.get(&to) {
                    let to_port = opposite_port(port);
                    if let Some(&Connection(_, from_node)) = map.get_output(to_port) {
                        return to_node == to && from_node == from;
                    }
                }
            }
        }

        false
    }

    /// Returns a view of the `IoBus` for the given node.
    pub fn view<'a>(&'a mut self, node: NodeId) -> IoBusView<'a> {
        assert!(self.nodes.get(&node).is_some());
        IoBusView::new(self, node)
    }

    /// Commits all outstanding writes and clears the write buffer.
    pub fn commit(&mut self) {
        for (i, &v) in self.writes.iter() {
            self.ports.insert(i, v);
        }

        self.writes.clear();
    }

    /// Send data on a given port for a node.
    fn write(&mut self, node: &NodeId, port: Port, value: isize) {
        if let Some(&Connection(index, _)) = self.get_output(node, port) {
            self.writes.insert(index, value);
            self.write_blocks.insert(*node, value);
        }
    }

    /// Check if an output port has been read for a node.
    fn is_blocked(&self, node: &NodeId) -> bool {
        self.write_blocks.get(node).is_some()
    }

    /// Receive data on a given port for a node. Whenever a node reads from an input, all of the
    /// outputs on the sending node are cleared.
    fn read(&mut self, node: &NodeId, port: Port) -> Option<isize> {
        if let Some(&Connection(index, node)) = self.get_input(node, port) {
            if let Some(val) = self.ports.remove(&index) {
                self.clear_outputs(&node);
                return Some(val);
            }
        }

        None
    }

    /// Get an input connection from a `PortMap`.
    fn get_input(&self, node: &NodeId, port: Port) -> Option<&Connection> {
        if let Some(map) = self.nodes.get(node) {
            map.get_input(port)
        } else {
            None
        }
    }

    /// Get an output connection from a `PortMap`.
    fn get_output(&self, node: &NodeId, port: Port) -> Option<&Connection> {
        if let Some(map) = self.nodes.get(node) {
            map.get_output(port)
        } else {
            None
        }
    }

    /// Create a new `PortMap`.
    fn insert_map(&mut self, node: NodeId) -> &mut Self {
        self.nodes.insert(node, PortMap::new());
        self
    }

    /// Check if a `PortMap` exists.
    fn map_exists(&self, node: &NodeId) -> bool {
        self.nodes.get(node).is_some()
    }

    /// Clear all of the output ports for a given node.
    fn clear_outputs(&mut self, node: &NodeId) {
        let to_clear = match self.nodes.get(node) {
            Some(map) => map.output_iter()
                            .map(|(_, &Connection(i, _))| { i })
                            .collect::<Vec<_>>(),
            None => Vec::new(),
        };

        for index in to_clear.iter() {
            self.ports.remove(index);
        }


        self.write_blocks.remove(node);
    }
}

/// Provides access to the `IoBus` for a single node. This ensures that nodes are only able to read
/// and write on ports that they are connected to.
#[derive(Debug)]
pub struct IoBusView<'a> {
    bus: &'a mut IoBus,
    node: usize,
}

impl<'a> IoBusView<'a> {
    /// Construct a new `IoBusView`.
    fn new(bus: &'a mut IoBus, node: usize) -> IoBusView<'a> {
        IoBusView {
            bus: bus,
            node: node,
        }
    }

    /// Receive data on a given port.
    pub fn read(&mut self, port: Port) -> Option<isize> {
        self.bus.read(&self.node, port)
    }

    /// Send data on a given port.
    pub fn write(&mut self, port: Port, value: isize) {
        self.bus.write(&self.node, port, value);
    }

    /// Check if an output port has been read.
    pub fn is_blocked(&self) -> bool {
        self.bus.is_blocked(&self.node)
    }
}

/// For a given node, this maps from an input or output port direction to the bus index containing
/// the data for that direction.
#[derive(Debug)]
struct PortMap {
    input: HashMap<Port, Connection>,
    output: HashMap<Port, Connection>,
}

impl PortMap {
    /// Construct a new, empty `PortMap`.
    fn new() -> PortMap {
        PortMap {
            input: HashMap::new(),
            output: HashMap::new(),
        }
    }

    /// Returns an iterator over the outputs.
    fn output_iter(&self) -> Iter<Port, Connection> {
        self.output.iter()
    }

    /// Set the input index for a given port direction. We also store the node that owns the
    /// corresponding output so that we can clear its outputs after a read.
    fn set_input(&mut self, port: Port, index: PortId, node: NodeId) {
        self.input.insert(port, Connection(index, node));
    }

    /// Get the input index for a given port direction. We also return the node that owns the
    /// corresponding output so that we can clear its outputs after a read.
    fn get_input(&self, port: Port) -> Option<&Connection> {
        self.input.get(&port)
    }

    /// Set the output index for a given port direction.
    fn set_output(&mut self, port: Port, index: PortId, node: NodeId) {
        self.output.insert(port, Connection(index, node));
    }

    /// Get the output index for a given port direction.
    fn get_output(&self, port: Port) -> Option<&Connection> {
        self.output.get(&port)
    }
}
