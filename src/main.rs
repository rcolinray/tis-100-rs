extern crate tis_100;

use std::io;
use std::env;
use std::thread;
use std::sync::mpsc::channel;
use std::sync::mpsc::TryRecvError::*;
use tis_100::save::load_save;
use tis_100::machine::Tis100;

fn main() {
    let args = env::args().collect::<Vec<_>>();

    if args.len() != 3 {
        println!("Please provide a save file to load using '-s'");
        return;
    }

    if let Some(save) = load_save(&args[2]) {
        let (in_tx, in_rx) = channel();
        let (out_tx, out_rx) = channel();

        // TIS-100 loop
        thread::spawn(move|| {
            let mut tis100 = Tis100::with_save(&save);

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

                thread::sleep_ms(1);
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

                thread::sleep_ms(1);
            }

            drop(out_rx);
        });

        // Console input loop
        let mut stdin = io::stdin();
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
}
