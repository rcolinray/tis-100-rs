//! Constructs for specifying TIS-100 puzzles.

use std::fs::File;
use std::path::Path;
use vec_map::VecMap;
use hlua::{Lua, LuaTable};
use hlua::functions_read::LuaFunction;
use save::Save;
use node::{Node, TestNode, BasicExecutionNode, DamagedExecutionNode, StackMemoryNode, TestInputNode, TestOutputNode, TestImageNode};
use machine::{NUM_NODES, INPUT_0, Tis100};

/// Used to seed the Lua random number generator.
const SEED_RANDOM_EXEC: &'static str = "math.randomseed(os.time())";

/// Constants for extracting the TIS-100 layout from the spec.
const LAYOUT_TABLE: &'static str = "layout";
const LAYOUT_FN: &'static str = "get_layout";
const LAYOUT_FN_EXEC: &'static str = "layout = get_layout()";

/// Constants for extracting the TIS-100 test streams from the spec.
const STREAMS_TABLE: &'static str = "streams";
const STREAMS_FN: &'static str = "get_streams";
const STREAMS_FN_EXEC: &'static str = "streams = get_streams()";
const STREAM_KIND_IDX: u32 = 1;
const STREAM_NAME_IDX: u32 = 2;
const STREAM_NODE_IDX: u32 = 3;
const STREAM_DATA_IDX: u32 = 4;

/// Enumerations for the stream kinds.
const STREAM_INPUT: u32 = 0;
const STREAM_OUTPUT: u32 = 1;
const STREAM_IMAGE: u32 = 2;

/// Enumerations for the tile kinds.
const TILE_COMPUTE: u32 = 0;
const TILE_MEMORY: u32 = 1;
const TILE_DAMAGED: u32 = 2;

/// The different kinds of nodes available to the spec.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
enum Tile {
    Compute,
    Memory,
    Damaged,
}

use self::Tile::*;

/// Intermediate representation of a test stream.
#[derive(Debug)]
struct Stream {
    kind: StreamKind,
    name: String,
    node: usize,
    data: Vec<isize>
}

/// The different kinds of streams available to the spec.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
enum StreamKind {
    Input,
    Output,
    Image,
}

use self::StreamKind::*;

/// An error that can be returned while loading a spec.
pub enum SpecError {
    SeedRandomFailed,
    ReadFileFailed,
    GetLayoutFailed,
    GetStreamsFailed,
}

use self::SpecError::*;

/// A specification for a TIS-100 puzzle. Specifications are Lua files that configure the layout,
/// inputs, and outputs for the TIS-100. At a minimum, a specification must provide the
/// `get_layout` and `get_streams` functions.
pub struct Spec {
    save: Save,
    layout: Vec<Tile>,
    streams: Vec<Stream>,
}

impl Spec {
    /// Load a `Spec` from a file.
    pub fn from_file(filename: &str, save: Save) -> Result<Spec, SpecError> {
        // Prepare the Lua context.
        let mut lua = Lua::new();
        lua.openlibs();

        if let Err(_) = lua.execute::<()>(SEED_RANDOM_EXEC) {
            return Err(SeedRandomFailed);
        }

        lua.set("STREAM_INPUT", STREAM_INPUT);
        lua.set("STREAM_OUTPUT", STREAM_OUTPUT);
        lua.set("STREAM_IMAGE", STREAM_IMAGE);
        lua.set("TILE_COMPUTE", TILE_COMPUTE);
        lua.set("TILE_MEMORY", TILE_MEMORY);
        lua.set("TILE_DAMAGED", TILE_DAMAGED);

        // Read and execute the spec file.
        if let Ok(file) = File::open(&Path::new(filename)) {
            if let Err(_) = lua.execute_from_reader::<(), _>(file) {
                return Err(ReadFileFailed);
            }
        } else {
            return Err(ReadFileFailed);
        }

        // Make sure that get_layout exists and can be called.
        if let None = lua.get::<LuaFunction<_>, _>(LAYOUT_FN) {
            return Err(GetLayoutFailed);
        }

        // FIXME: Figure out how to return a LuaTable from a LuaFunction call.
        //        For now we call the get_layout function and save the result table to a variable.
        if let Err(_) = lua.execute::<()>(LAYOUT_FN_EXEC) {
            return Err(GetLayoutFailed);
        }

        // Read the layout from Lua.
        let mut layout = Vec::new();
        if let Some(mut layout_table) = lua.get::<LuaTable<_>, _>(LAYOUT_TABLE) {
            for (_, v) in layout_table.iter::<u32, u32>().filter_map(|e| e) {
                match v {
                    TILE_COMPUTE => layout.push(Compute),
                    TILE_MEMORY => layout.push(Memory),
                    TILE_DAMAGED => layout.push(Damaged),
                    _ => return Err(GetLayoutFailed),
                };
            }

            if layout.len() != NUM_NODES {
                return Err(GetLayoutFailed);
            }
        }

        // Make sure that get_streams exists and can be called.
        if let None = lua.get::<LuaFunction<_>, _>(STREAMS_FN) {
            return Err(GetStreamsFailed);
        }

        // FIXME: Figure out how to return a LuaTable from a LuaFunction call.
        //        For now we call the get_streams function and save the result table to a variable.
        if let Err(_) = lua.execute::<()>(STREAMS_FN_EXEC) {
            return Err(GetStreamsFailed);
        }

        // Read the streams from Lua.
        let mut streams = Vec::new();
        if let Some(mut streams_table) = lua.get::<LuaTable<_>, _>(STREAMS_TABLE) {
            // FIXME: Figure out how to iterate over a table of tables.
            //        For now, we can only have 8 total inputs and outputs, so just try each index.
            for index in 1..9 {
                // Each stream is a table with the following format:
                // 1: kind (input, output, image)
                // 2: name
                // 3: node the stream is connected to
                // 4: data stream
                if let Some(mut stream_table) = streams_table.get::<LuaTable<_>, _>(index) {
                    let kind = match stream_table.get::<u32, _>(STREAM_KIND_IDX) {
                        Some(STREAM_INPUT) => Input,
                        Some(STREAM_OUTPUT) => Output,
                        Some(STREAM_IMAGE) => Image,
                        _ => return Err(GetStreamsFailed),
                    };

                    let name = match stream_table.get::<String, _>(STREAM_NAME_IDX) {
                        Some(name) => name,
                        None => return Err(GetStreamsFailed),
                    };

                    let node = match stream_table.get::<u32, _>(STREAM_NODE_IDX) {
                        Some(node) => node as usize,
                        None => return Err(GetStreamsFailed),
                    };

                    let data = match stream_table.get::<LuaTable<_>, _>(STREAM_DATA_IDX) {
                        Some(mut data_table) => {
                            let mut data = Vec::new();
                            for (_, v) in data_table.iter::<u32, i32>().filter_map(|e| e) {
                                data.push(v as isize);
                            }
                            data
                        },
                        None => return Err(GetStreamsFailed),
                    };

                    streams.push(Stream {
                        kind: kind,
                        name: name,
                        node: node,
                        data: data,
                    });
                } else {
                    break;
                }
            }
        }

        Ok(Spec {
            save: save,
            layout: layout,
            streams: streams,
        })
    }

    /// Configure a `Tis100` instance using the spec.
    pub fn setup(&mut self, cpu: &mut Tis100) {
        for (index, &tile) in self.layout.iter().enumerate() {
            let node: Box<Node> = match tile {
                Compute => match self.save.get(index) {
                    Some(prog) => Box::new(BasicExecutionNode::with_program(prog.clone())),
                    None => Box::new(BasicExecutionNode::new()),
                },
                Memory => Box::new(StackMemoryNode::new()),
                Damaged => Box::new(DamagedExecutionNode),
            };

            cpu.add_node(index, node);
        }

        // Test inputs are added as regular nodes since we probably don't need to interact with
        // them after they are set up.
        for stream in self.streams.iter() {
            if let Input = stream.kind {
                cpu.add_node(stream.node + INPUT_0, Box::new(TestInputNode::with_data(&stream.data)));
            }
        }
    }

    /// Get the test output nodes used by the spec.
    pub fn tests(&self) -> VecMap<Box<TestNode>> {
        let mut tests: VecMap<Box<TestNode>> = VecMap::new();

        for stream in self.streams.iter() {
            match stream.kind {
                Input => (),
                Output => {
                    tests.insert(stream.node, Box::new(TestOutputNode::with_data(&stream.data)));
                },
                Image => {
                    tests.insert(stream.node, Box::new(TestImageNode::with_data(&stream.data, 30, 18)));
                },
            };
        }

        tests
    }
}
