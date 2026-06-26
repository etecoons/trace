use std::io::{self, BufRead};

fn main() {
    // Clear screen
    print!("\x1B[2J\x1B[1;1H");

    println!("\x1B[1;31m");
    println!("       .·'  `'·.");
    println!("    .-'  (O)(O)  '-.");
    println!("   /   .-'  ''  '-.   \\");
    println!("  |   /   .---.    \\   |");
    println!("  |  |   /  _  \\    |  |");
    println!("   \\  \\  | (_) |   /  /");
    println!("    '-. \\ \\___/  .-'");
    println!("       `'·.|||.·'");
    println!("\x1B[0m");

    println!("\x1B[1;32m=== UBERMETROID COMPANION SYSTEMS ===\x1B[0m");
    println!("\x1B[1;33mTerminal Access Mode:\x1B[0m Restricted (Containerized Nix Shell)");
    println!("\x1B[1;36mSuit Energy:\x1B[0m 99/99");
    println!("\x1B[1;36mMissiles:\x1B[0m 255/255");
    println!("\x1B[1;36mLocation:\x1B[0m Zebes Orbit / Docker Sandbox");
    println!("\x1B[1;35mStatus:\x1B[0m Safe & Isolated");
    println!("\nPress \x1B[1;37m[Enter]\x1B[0m to close console connection...");

    let stdin = io::stdin();
    let mut buffer = String::new();
    let _ = stdin.lock().read_line(&mut buffer);
}
