extern crate tis_100;

use std::env;
use std::thread;
use tis_100::save::{load_save, pretty_print_errors};
use tis_100::save::LoadSaveError::*;
use tis_100::spec::Spec;
use tis_100::spec::SpecError::*;
use tis_100::machine::Puzzle;
use tis_100::node::TestState::*;

const USAGE: &'static str = "TIS-100 Puzzle Emulator\n\nUsage:\n    puzzle <spec.lua> <save.txt>";

fn main() {
    let args = env::args().collect::<Vec<_>>();

    // Check args for spec and save filenames
    if args.len() < 3 || args[1] == "-h" || args[1] == "--help" {
        println!("{}", USAGE);
        return;
    }

    // Load and parse the save file
    let save = match load_save(&args[2]) {
        Ok(save) => save,
        Err(ParseFailed(errs)) => {
            println!("Could not parse save file");
            pretty_print_errors(errs);
            return;
        },
        Err(_) => panic!("Could not load save file"),
    };

    let mut spec = match Spec::from_file(&args[1], save) {
        Ok(spec) => spec,
        Err(SeedRandomFailed) => panic!("Could not seed random number generator"),
        Err(ReadFileFailed) => panic!("Could not load spec file"),
        Err(GetLayoutFailed) => panic!("Could not load layout from spec file"),
        Err(GetStreamsFailed) => panic!("Could not load streams from spec file"),
    };

    let mut puzzle = Puzzle::from_spec(&mut spec);
    loop {
        puzzle.step();

        let state = puzzle.state();

        if state != Testing {
            if state == Passed {
                println!("PASSED");
            } else {
                println!("FAILED");
            }

            println!("CYCLES: {}", puzzle.cycles());
            break;
        }

        if puzzle.is_deadlocked() {
            println!("DEADLOCK");
            break;
        }

        thread::sleep_ms(1);
    }

}
