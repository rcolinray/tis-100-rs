use std::collections::VecMap;
use std::io::Read;
use std::fs::File;
use core::Program;
use parse::{parse_program, ProgramErrors};
use error::pretty_print_save_errors;

pub type Save = VecMap<Program>;

pub type SaveErrors = VecMap<ProgramErrors>;

pub fn load_save(filename: &str) -> Option<Save> {
    if let Ok(mut file) = File::open(filename) {
        let mut src = String::new();
        if let Ok(_) = file.read_to_string(&mut src) {
            match parse_save(&src) {
                Ok(save) => Some(save),
                Err(errors) => {
                    pretty_print_save_errors(errors);
                    None
                }
            }
        } else {
            None
        }
    } else {
        None
    }
}

fn parse_save(src: &str) -> Result<Save, SaveErrors> {
    let mut save = VecMap::new();
    let mut errors = VecMap::new();
    let mut prog_src = String::new();
    let mut num = None;

    for line in src.lines() {
        if line.starts_with("@") {
            if let Some(n) = num {
                match parse_program(&prog_src) {
                    Ok(prog) => {
                        save.insert(n, prog);
                    },
                    Err(errs) => {
                        errors.insert(n, errs);
                    },
                };
                prog_src.clear();
            }

            num = line.chars()
                .skip(1)
                .take_while(|c| c.is_numeric())
                .collect::<String>()
                .parse::<usize>()
                .ok();
        } else if num != None {
            prog_src.push_str(line);
        }
    }

    if let Some(n) = num {
        if prog_src.len() > 0 {
            match parse_program(&prog_src) {
                Ok(prog) => {
                    save.insert(n, prog);
                },
                Err(errs) => {
                    errors.insert(n, errs);
                }
            };
            prog_src.clear();
        }
    }

    if errors.len() > 0 {
        Err(errors)
    } else {
        Ok(save)
    }
}
