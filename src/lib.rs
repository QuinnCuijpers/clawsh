pub mod handle_command;
pub mod parser;
mod invoke;
mod completion;
mod util;
mod commands;

pub use crate::completion::TrieCompleter;
pub use crate::commands::BUILTIN_COMMANDS as BUILTIN_COMMANDS;