use std::process::{Command, Stdio};

#[test]
fn check_fzf_command_output() {
    let output = Command::new(env!(concat!("CARGO_BIN_EXE_", clap::crate_name!())))
        .env("JQ_REPL_TEST", "true")
        .arg("--history-file")
        .arg("/tmp/jq_repl_history")
        .arg("--config")
        .arg("./tests/config.toml")
        .arg("--show-fzf-command")
        .arg("-n")
        .arg("./tests/foo bar.json")
        .stdin(Stdio::inherit())
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();

    insta::assert_snapshot!(stdout);
}
