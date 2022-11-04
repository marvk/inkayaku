use std::cell::RefCell;
use std::io::{Error, stdin};

use marvk_chess_engine_lib::inkayaku::Inkayaku;
use marvk_chess_uci::uci::console::{ConsoleUciRx, ConsoleUciTx};
use marvk_chess_uci::uci::Engine;

fn main() {
    let tx = ConsoleUciTx::new(|command| println!("{}", command));
    let engine = RefCell::new(Inkayaku::new(tx));
    let rx = ConsoleUciRx::new(read_line, move |command_result| {
        if let Ok(command) = command_result {
            engine.borrow_mut().accept(command);
        }
    });

    rx.start();
}

fn read_line() -> Result<String, Error> {
    let mut result = String::new();
    stdin().read_line(&mut result).map(|_| result)
}
