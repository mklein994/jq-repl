use std::io::prelude::*;

const LIMIT: usize = 1_000;

fn main() {
    let mut buf = String::new();
    std::io::stdin().read_to_string(&mut buf).unwrap();
    let count = buf.chars().count();

    match count {
        x if x < LIMIT - 100 => {}
        x if x < LIMIT - 25 => print!("\x1b[2;37m{count}/{LIMIT}\x1b[0m"), // dim gray
        x if x < LIMIT - 10 => print!("{count}/{LIMIT}"),                  // plain
        x if x < LIMIT - 3 => print!("\x1b[0;33m{count}/{LIMIT}\x1b[0m"),  // yellow
        x if x < LIMIT => print!("\x1b[0;31m{count}/{LIMIT}\x1b[0m"),      // red
        _ => print!("\x1b[1;31m{count}/{LIMIT}\x1b[0m"),                   // bold red
    }
}
