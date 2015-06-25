//! Functions for loading TIS-100 assembly code from save files.

use std::collections::VecMap;
use std::io::Read;
use std::fs::File;
use core::Program;
use parse::{parse_program, ProgramErrors};

/// Programs that are assigned to specific nodes in a TIS-100.
pub type Save = VecMap<Program>;

/// A list of errors returned when parsing programs for each node.
pub type SaveErrors = VecMap<ProgramErrors>;

/// An error returned when loading a safe from a file.
pub enum LoadSaveError {
    ParseFailed(SaveErrors),
    LoadFailed,
}

use self::LoadSaveError::*;

/// Load a `Save` from a file.
pub fn load_save(filename: &str) -> Result<Save, LoadSaveError> {
    match File::open(filename) {
        Ok(mut file) => {
            let mut src = String::new();
            match file.read_to_string(&mut src) {
                Ok(_) => match parse_save(&src) {
                    Ok(prog) => Ok(prog),
                    Err(errs) => Err(ParseFailed(errs)),
                },
                Err(_) => Err(LoadFailed),
            }
        },
        Err(_) => Err(LoadFailed),
    }
}

/// Parse the text of a TIS-100 save file into a map from node numbers to programs.
pub fn parse_save(src: &str) -> Result<Save, SaveErrors> {
    let mut save = VecMap::new();
    let mut errors = VecMap::new();

    // Skip the first result since it will be empty.
    for src in src.split("@").skip(1) {
        let maybe_num = src.chars()
            .take_while(|c| c.is_numeric())
            .collect::<String>()
            .parse::<usize>()
            .ok();

        if let Some(num) = maybe_num {
            // Skip the first line since it has the node number.
            let prog_src = src.chars()
                .skip_while(|&c| c != '\n')
                .skip(1)
                .collect::<String>();

            match parse_program(&prog_src) {
                Ok(prog) => {
                    save.insert(num, prog);
                },
                Err(errs) => {
                    errors.insert(num, errs);
                }
            }
        }
    }

    if errors.len() > 0 {
        Err(errors)
    } else {
        Ok(save)
    }
}

/// Pretty print errors from parsing a save file.
pub fn pretty_print_errors(save_errors: SaveErrors) {
    for (node_num, ref errors) in save_errors.iter() {
        for &(line_num, ref error) in errors.iter() {
            println!("Node {}: Line {}: {}\n", node_num, line_num, error);
        }
    }
}

#[test]
fn test_parse_save() {
    let save = parse_save("@1\nADD 1\n@2\nADD 1\n").unwrap();
    assert_eq!(save.len(), 2);
}
