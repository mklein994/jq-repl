use std::process::ExitCode;

fn main() -> ExitCode {
    if let Err(err) = jq_repl::run() {
        eprintln!("{err}");
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
