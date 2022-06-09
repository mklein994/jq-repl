use jq_repl::Error;

fn main() {
    if let Err(err) = jq_repl::run() {
        if let Error::Fzf(status) = err {
            if let Some(code) = status.code() {
                if !status.success() {
                    std::process::exit(code);
                }
            }
        }
        eprintln!("{err}");
        std::process::exit(1);
    }
}
