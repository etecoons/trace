use std::io::{self, BufRead};

fn main() {
    // Clear screen
    print!("\x1B[2J\x1B[1;1H");

    println!(r#" _______________________________________
/ I'm sorry, Dave. I'm afraid Nix won't \
\ let me do that.                       /
 ---------------------------------------
        \   ^__^
         \  (oo)\_______
            (__)\       )\/\
                ||----w |
                ||     ||"#);

    println!("\x1B[1;31m\nSystem Alert: Console Access is UNAUTHORIZED.\x1B[0m");
    println!("This application is running inside a secure, read-only Nix container.");
    println!("Direct shell access is disabled for environment isolation and security.");
    println!("\nPress \x1B[1;37m[Enter]\x1B[0m to close connection...");

    let stdin = io::stdin();
    let mut buffer = String::new();
    let _ = stdin.lock().read_line(&mut buffer);
}
