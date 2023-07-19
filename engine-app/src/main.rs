use std::cell::RefCell;
use std::io::{Error, stdin};
use std::sync::Arc;


use marvk_chess_engine_lib::inkayaku::Inkayaku;
use marvk_chess_uci::uci::console::{ConsoleUciRx, ConsoleUciTx};
use marvk_chess_uci::uci::console::ConsoleUciRxError::CommandParseError;
use marvk_chess_uci::uci::{Engine};
use marvk_chess_uci::uci::parser::ParserError::UnknownCommand;
use marvk_chess_uci::uci::UciCommand::SetDebug;

fn main() {
    print_ln("Inkayaku by Marvin Kuhnke (see https://github.com/marvk/rust-chess)");
    let tx = Arc::new(ConsoleUciTx::new(print_ln, print_err));
    let engine = RefCell::new(Inkayaku::new(tx.clone()));
    let on_command = move |command_result| {
        match command_result {
            Ok(command) => {
                if let SetDebug {debug}  = command {
                    tx.set_debug(debug);
                }
                engine.borrow_mut().accept(command);
            },
            Err(CommandParseError(UnknownCommand(command))) => eprintln!("Unknown Command: {}", command),
            Err(error) => eprintln!("Failed to parse command: {:?}", error),
        }
    };
    let rx = ConsoleUciRx::new(read_line, on_command);

    rx.start();
}

fn read_line() -> Result<String, Error> {
    let mut result = String::new();
    stdin().read_line(&mut result).map(|_| result)
}

fn print_ln(line: &str) {
    println!("{}", line)
}

fn print_err(line: &str) {
    eprintln!("DEBUG: {}", line)
}
