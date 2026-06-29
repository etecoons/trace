use std::io::{self, BufRead};
use std::thread;
use std::time::Duration;

fn main() {
    shared_backend::security::print_unauthorized_console_message();

    let stdin = io::stdin();
    let mut handle = stdin.lock();
    loop {
        let mut buffer = String::new();
        match handle.read_line(&mut buffer) {
            Ok(0) => {
                // Stdin closed (EOF), sleep to keep the console window open
                thread::sleep(Duration::from_secs(3600));
            }
            Ok(_) => {
                // User pressed enter, reprint message
                shared_backend::security::print_unauthorized_console_message();
            }
            Err(_) => {
                thread::sleep(Duration::from_secs(3600));
            }
        }
    }
}
