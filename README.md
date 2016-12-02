# tis-100-rs [![Build Status](https://travis-ci.org/rcolinray/tis-100-rs.svg?branch=master)](https://travis-ci.org/rcolinray/tis-100-rs) [![Latest Version](https://img.shields.io/crates/v/tis-100.svg)](https://crates.io/crates/tis-100)

An emulator for the TIS-100 written in Rust.

## Binaries

This project includes two binaries: `sandbox`, which implements the TIS-100 *Simple Sandbox* puzzle,
and `puzzle`, which can execute arbitrary puzzles given a spec file and a save file.

```
TIS-100 Sandbox Emulator

Usage:
    sandbox <save.txt>
```

```
TIS-100 Puzzle Emulator

Usage:
    puzzle <spec.lua> <save.txt>
```

## Library

If you want to embed a TIS-100 emulator in your Rust project, simply add the following dependency to your `Cargo.toml`:

```toml
[dependencies]
tis-100 = "0.2.2"
```

Example:

```rust
extern crate tis_100;

use tis_100::save::parse_save;
use tis_100::machine::Sandbox;

// This program reads the value from the console and simply passes it to the console output.
let src = "@1\nMOV UP DOWN\n@5\nMOV UP DOWN\n@9\nMOV UP RIGHT\n@10\nMOV LEFT DOWN\n";

let save = parse_save(src).unwrap();
let mut sandbox = Sandbox::from_save(&save);

sandbox.write_console(42);

for _ in 0..5 {
    sandbox.step();
}

assert_eq!(sandbox.read_console(), Some(42));
```
