use std::io::prelude::*;

fn main() {
    let mut buf = String::new();
    std::io::stdin().read_to_string(&mut buf).unwrap();
    let count = buf.chars().count();

    match count {
        x if x < 200 => {}
        x if x < 275 => print!("\x1b[2;37m{count}/300\x1b[0m"), // dim gray
        x if x < 290 => print!("{count}/300"),                  // plain
        x if x < 297 => print!("\x1b[0;33m{count}/300\x1b[0m"), // yellow
        x if x < 300 => print!("\x1b[0;31m{count}/300\x1b[0m"), // red
        _ => print!("\x1b[1;31m{count}/300\x1b[0m"),            // bold red
    };
}
