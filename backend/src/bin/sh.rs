use std::io::{self, BufRead};

fn main() {
    shared_backend::security::print_unauthorized_console_message();

    let stdin = io::stdin();
    let mut buffer = String::new();
    let _ = stdin.lock().read_line(&mut buffer);
}
