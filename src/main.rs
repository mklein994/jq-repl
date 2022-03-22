fn main() {
    match jq_repl::run() {
        Ok(exit_status) => {
            if let Some(code) = exit_status.and_then(|exit| exit.code()) {
                std::process::exit(code);
            }
        }
        Err(err) => {
            eprintln!("{:?}", err);
            std::process::exit(1);
        }
    }
}
