extern crate tis_100;

use std::io;
use std::env;
use std::thread;
use std::time;
use std::sync::mpsc::channel;
use std::sync::mpsc::TryRecvError::*;
use tis_100::save::{load_save, pretty_print_errors};
use tis_100::save::LoadSaveError::*;
use tis_100::machine::Sandbox;

const USAGE: &'static str = "TIS-100 Sandbox Emulator\n\nUsage:\n    sandbox <save.txt>";

fn main() {
    let args = env::args().collect::<Vec<_>>();

    // Check args for save filename
    if args.len() == 1 || args[1] == "-h" || args[1] == "--help" {
        println!("{}", USAGE);
        return;
    }

    // Load and parse the save file
    let save = match load_save(&args[1]) {
        Ok(save) => save,
        Err(ParseFailed(errs)) => {
            println!("Could not parse save file");
            pretty_print_errors(errs);
            return;
        },
        Err(_) => {
            println!("Could not load save file");
            return;
        }
    };

    // Channels for communicating from the command-line to the TIS-100
    let (in_tx, in_rx) = channel();
    let (out_tx, out_rx) = channel();

    // TIS-100 loop
    thread::spawn(move|| {
        let mut tis100 = Sandbox::from_save(&save);

        loop {
            match in_rx.try_recv() {
                Ok(val) => tis100.write_console(val),
                Err(Disconnected) => break,
                _ => (),
            };

            tis100.step();

            if let Some(val) = tis100.read_console() {
                if let Err(_) = out_tx.send(val) {
                    break;
                }
            }

         thread::sleep(time::Duration::from_millis(1));
        }

        drop(in_rx);
        drop(out_tx);
    });

    // Console output loop
    thread::spawn(move|| {
        loop {
            match out_rx.try_recv() {
                Ok(val) => println!("> {}", val),
                Err(Disconnected) => break,
                Err(_) => (),
            };

         thread::sleep(time::Duration::from_millis(1));
        }

        drop(out_rx);
    });

    // Console input loop
    let stdin = io::stdin();
    loop {
        let mut input = String::new();
        match stdin.read_line(&mut input) {
            Ok(_) => if let Ok(val) = str::parse::<isize>(input.trim_right()) {
                if let Err(_) = in_tx.send(val) {
                    break;
                }
            },
            Err(_) => break,
        }
    }

    drop(in_tx);
}
