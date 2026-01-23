use std::io::Read;

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/commands.txt");
    let commands_contents = std::fs::read_to_string(path)?;
    let commands = commands_contents.lines().collect::<Vec<_>>();

    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input)?;

    let (prefix, search_term) = match input
        .trim_end()
        .rsplit_once(|c: char| c.is_ascii_whitespace())
    {
        Some((start, "")) => (None, start),
        Some((start, end)) => (Some(start), end),
        None => (None, input.trim_end()),
    };

    let candidates: Vec<_> = commands
        .iter()
        .filter(|x| x.starts_with(search_term))
        .collect();

    if candidates.is_empty() {
        print!("{input}")
    } else if candidates.len() == 1 {
        let command = candidates[0];
        if let Some(prefix) = prefix {
            print!("{prefix} {command}");
        } else {
            print!("{command}");
        }
    } else {
        eprintln!("{candidates:?}");
        print!("{input}");
    };

    Ok(())
}
