use std::cell::RefCell;
use std::io::stdin;
use std::sync::Arc;

use inkayaku_engine_core::engine::Engine;
use inkayaku_uci::{UciEngine, UciTx};
use inkayaku_uci::console::{ConsoleUciRx, ConsoleUciTx};
use inkayaku_uci::console::ConsoleUciRxError::CommandParseError;
use inkayaku_uci::parser::ParserError::UnknownCommand;
use inkayaku_uci::UciCommand::SetDebug;

#[cfg(feature = "debug")]
const DEBUG_DEFAULT: bool = true;
#[cfg(not(feature = "debug"))]
const DEBUG_DEFAULT: bool = false;

fn main() {
    let tx = Arc::new(ConsoleUciTx::new(print_ln, print_err, DEBUG_DEFAULT));
    if DEBUG_DEFAULT { tx.debug("DEBUG ENABLED") }
    print_ln("Inkayaku by Marvin Kuhnke (see https://github.com/marvk/rust-chess)");
    let engine = RefCell::new(Engine::new(tx.clone(), DEBUG_DEFAULT));
    let on_command = |command_result| {
        match command_result {
            Ok(command) => {
                if let SetDebug { debug } = command {
                    tx.set_debug(debug);
                }
                engine.borrow_mut().accept(command);
            }
            Err(CommandParseError(UnknownCommand(command))) => eprintln!("Unknown Command: {}", command),
            Err(error) => eprintln!("Failed to parse command: {:?}", error),
        }
    };
    let rx = ConsoleUciRx::new(read_line, on_command);

    rx.start();
}

fn read_line() -> Result<String, std::io::Error> {
    let mut result = String::new();
    stdin().read_line(&mut result)?;
    Ok(result)
}

fn print_ln(line: &str) {
    println!("{}", line);
}

fn print_err(line: &str) {
    eprintln!("DEBUG: {}", line);
}
