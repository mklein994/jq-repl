use std::process::ExitCode;

fn main() -> ExitCode {
    match jq_repl::run() {
        Ok(exit_status) => match exit_status {
            Some(status) if status.success() => ExitCode::SUCCESS,
            Some(status) => {
                eprintln!("{status}");
                ExitCode::FAILURE
            }
            None => ExitCode::SUCCESS,
        },
        Err(err) => {
            eprintln!("{:?}", err);
            ExitCode::FAILURE
        }
    }
}
