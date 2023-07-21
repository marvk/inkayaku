use std::sync::{Mutex};
use std::sync::mpsc::Sender;

use crate::uci::{Info, ProtectionMessage, UciMove, UciTx, UciTxCommand};

pub struct CommandUciTx {
    command_consumer: Mutex<Sender<UciTxCommand>>,
}

impl CommandUciTx {
    fn send(&self, command: UciTxCommand) {
        self.command_consumer.lock().unwrap().send(command).unwrap();
    }
    pub fn new(command_consumer: Sender<UciTxCommand>) -> Self {
        // TODO Spawn channel inside (?)

        Self { command_consumer: Mutex::new(command_consumer) }
    }
}

impl UciTx for CommandUciTx {
    fn id_name(&self, name: &str) {
        self.send(UciTxCommand::IdName { name: name.to_string() });
    }

    fn id_author(&self, author: &str) {
        self.send(UciTxCommand::IdAuthor { author: author.to_string() });
    }

    fn uci_ok(&self) {
        self.send(UciTxCommand::Ok);
    }

    fn ready_ok(&self) {
        self.send(UciTxCommand::ReadyOk);
    }

    fn best_move(&self, uci_move: Option<UciMove>) {
        self.send(UciTxCommand::BestMove { uci_move });
    }

    fn best_move_with_ponder(&self, uci_move: &UciMove, ponder_uci_move: &UciMove) {
        self.send(UciTxCommand::BestMoveWithPonder { uci_move: uci_move.clone(), ponder_uci_move: ponder_uci_move.clone() });
    }

    fn copy_protection(&self, copy_protection: ProtectionMessage) {
        self.send(UciTxCommand::CopyProtection { copy_protection });
    }

    fn registration(&self, registration: ProtectionMessage) {
        self.send(UciTxCommand::Registration { registration });
    }

    fn info(&self, info: &Info) {
        self.send(UciTxCommand::Info { info: info.clone() });
    }

    fn option_check(&self, name: &str, default: bool) {
        self.send(UciTxCommand::OptionCheck { name: name.to_string(), default });
    }

    fn option_spin(&self, name: &str, default: i32, min: i32, max: i32) {
        self.send(UciTxCommand::OptionSpin { name: name.to_string(), default, min, max });
    }

    fn option_combo(&self, name: &str, default: &str, vars: &[&str]) {
        self.send(UciTxCommand::OptionCombo { name: name.to_string(), default: default.to_string(), vars: vars.iter().map(|s| s.to_string()).collect() });
    }

    fn option_button(&self, name: &str) {
        self.send(UciTxCommand::OptionButton { name: name.to_string() });
    }

    fn option_string(&self, name: &str, default: &str) {
        self.send(UciTxCommand::OptionString { name: name.to_string(), default: default.to_string() });
    }

    fn debug(&self, message: &str) {
        self.send(UciTxCommand::Debug { message: message.to_string() });
    }
}
