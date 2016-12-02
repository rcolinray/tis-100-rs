//! A TIS-100 emulator.
//!
//! # Example
//!
//! ```
//! use tis_100::save::parse_save;
//! use tis_100::machine::Sandbox;
//!
//! // This program reads the value from the console and simply passes it to the console output.
//! let src = "@1\nMOV UP DOWN\n@5\nMOV UP DOWN\n@9\nMOV UP RIGHT\n@10\nMOV LEFT DOWN\n";
//!
//! let save = parse_save(src).unwrap();
//! let mut sandbox = Sandbox::from_save(&save);
//!
//! sandbox.write_console(42);
//!
//! for _ in 0..5 {
//!     sandbox.step();
//! }
//!
//! assert_eq!(sandbox.read_console(), Some(42));
//! ```

extern crate hlua;
extern crate vec_map;

pub mod core;
pub mod lex;
pub mod parse;
pub mod io;
pub mod node;
pub mod image;
pub mod save;
pub mod spec;
pub mod machine;
