mod cli;
mod data;
mod doctor;
mod process;
mod status;

use std::env;

pub const APP_NAME: &str = "Trace";
pub const ENV_PREFIX: &str = "TRACE";
pub const DB_FILE_NAME: &str = "";

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        let cmd = args[1].to_lowercase();
        if cmd == "tui" {
            eprintln!(
                "The interactive TUI console has been removed. Use 'sh help' to see available CUI commands."
            );
            std::process::exit(2);
        }
        cli::handle_cli_args(&args);
    } else {
        cli::print_help();
    }
}
