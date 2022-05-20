fn main() {
    if let Err(err) = jq_repl::run() {
        eprintln!("{err}");
        std::process::exit(1);
    }
}
